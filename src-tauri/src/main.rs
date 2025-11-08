#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod scanner;
mod tags;
mod genres;
mod genre_cache;
mod metadata;
mod processor;
mod audible;
mod cache;
mod progress;
mod tag_inspector;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[tauri::command]
fn get_config() -> config::Config {
    config::load_config().unwrap_or_default()
}

#[tauri::command]
fn save_config(config: config::Config) -> Result<(), String> {
    config::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_library(
    _window: tauri::Window,
    paths: Vec<String>,
) -> Result<serde_json::Value, String> {
    let config = config::load_config().map_err(|e| e.to_string())?;
    
    let api_key = if config.openai_api_key.is_empty() {
        None
    } else {
        Some(config.openai_api_key)
    };
    
    let config = config::load_config().map_err(|e| e.to_string())?;
    
    let groups = scanner::scan_directory(
        &paths[0], 
        api_key,
        config.skip_unchanged,
        None
    )
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "groups": groups
    }))
}

#[derive(Debug, Deserialize)]
struct WriteRequest {
    file_ids: Vec<String>,
    files: HashMap<String, FileData>,
    backup: bool,
}

#[derive(Debug, Deserialize)]
struct FileData {
    path: String,
    changes: HashMap<String, scanner::FieldChange>,
}

#[tauri::command]
async fn write_tags(request: WriteRequest) -> Result<tags::WriteResult, String> {
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for file_id in &request.file_ids {
        if let Some(file_data) = request.files.get(file_id) {
            match tags::write_file_tags(&file_data.path, &file_data.changes, request.backup).await {
                Ok(_) => success += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(tags::WriteError {
                        file_id: file_id.clone(),
                        path: file_data.path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }
    }
    
    Ok(tags::WriteResult { success, failed, errors })
}

#[tauri::command]
async fn test_abs_connection(config: config::Config) -> Result<ConnectionTest, String> {
    if config.abs_base_url.is_empty() {
        return Ok(ConnectionTest {
            success: false,
            message: "No URL configured".to_string(),
        });
    }
    
    Ok(ConnectionTest {
        success: true,
        message: format!("Connected to {}", config.abs_base_url),
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionTest {
    success: bool,
    message: String,
}

#[tauri::command]
async fn inspect_file_tags(file_path: String) -> Result<tag_inspector::RawTags, String> {
    tag_inspector::inspect_file_tags(&file_path).map_err(|e| e.to_string())
}

mod audible_auth;

// ============================================================================
// MAINTENANCE COMMANDS
// ============================================================================

#[tauri::command]
async fn clear_cache() -> Result<String, String> {
    cache::MetadataCache::new()
        .map_err(|e| e.to_string())?
        .clear()
        .map_err(|e| e.to_string())?;
    Ok("Cache cleared successfully".to_string())
}

#[tauri::command]
async fn restart_abs_docker() -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("docker")
        .args(&["restart", "audiobookshelf"])
        .output()
        .map_err(|e| format!("Failed to execute docker command: {}", e))?;
    
    if output.status.success() {
        Ok("Container restarted successfully".to_string())
    } else {
        Err(format!("Docker restart failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

#[tauri::command]
async fn force_abs_rescan() -> Result<String, String> {
    let config = config::load_config().map_err(|e| e.to_string())?;
    
    let client = reqwest::Client::new();
    let url = format!("{}/api/libraries/{}/scan", config.abs_base_url, config.abs_library_id);
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.abs_api_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if response.status().is_success() {
        Ok("Library rescan triggered".to_string())
    } else {
        Err(format!("Failed to trigger rescan: {}", response.status()))
    }
}

#[tauri::command]
async fn clear_abs_cache() -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("docker")
        .args(&["exec", "audiobookshelf", "rm", "-rf", "/config/cache/*"])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;
    
    if output.status.success() {
        Ok("Cache cleared successfully".to_string())
    } else {
        Err(format!("Failed to clear cache: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

// ============================================================================
// GENRE MANAGEMENT
// ============================================================================

#[derive(Debug, Deserialize)]
struct LibraryFilterData {
    genres: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LibraryItem {
    id: String,
    media: Media,
}

#[derive(Debug, Deserialize)]
struct Media {
    metadata: ItemMetadata,
}

#[derive(Debug, Deserialize)]
struct ItemMetadata {
    genres: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct LibraryItemsResponse {
    results: Vec<LibraryItem>,
}

#[tauri::command]
async fn clear_all_genres() -> Result<String, String> {
    let config = config::load_config().map_err(|e| e.to_string())?;
    
    if config.abs_base_url.is_empty() || config.abs_api_token.is_empty() || config.abs_library_id.is_empty() {
        return Err("AudiobookShelf not configured. Please set Base URL, API Token, and Library ID in Settings.".to_string());
    }
    
    let client = reqwest::Client::new();
    
    // Step 1: Get all genres from the library filter data (the dropdown)
    let filter_url = format!("{}/api/libraries/{}/filterdata", config.abs_base_url, config.abs_library_id);
    let filter_response = client
        .get(&filter_url)
        .header("Authorization", format!("Bearer {}", config.abs_api_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch filter data: {}", e))?;
    
    if !filter_response.status().is_success() {
        return Err(format!("Failed to fetch filter data: {}", filter_response.status()));
    }
    
    let filter_data: LibraryFilterData = filter_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse filter data: {}", e))?;
    
    let all_dropdown_genres = filter_data.genres;
    
    // Step 2: Get all genres actually used by books
    let items_url = format!("{}/api/libraries/{}/items?limit=1000", config.abs_base_url, config.abs_library_id);
    let items_response = client
        .get(&items_url)
        .header("Authorization", format!("Bearer {}", config.abs_api_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch library items: {}", e))?;
    
    if !items_response.status().is_success() {
        return Err(format!("Failed to fetch library items: {}", items_response.status()));
    }
    
    let items: LibraryItemsResponse = items_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse library items: {}", e))?;
    
    // Collect all genres currently used by books
    let mut used_genres: HashSet<String> = HashSet::new();
    for item in items.results {
        if let Some(genres) = item.media.metadata.genres {
            for genre in genres {
                used_genres.insert(genre);
            }
        }
    }
    
    // Step 3: Find unused genres (in dropdown but not used by any book)
    let unused_genres: Vec<String> = all_dropdown_genres
        .into_iter()
        .filter(|g| !used_genres.contains(g))
        .collect();
    
    if unused_genres.is_empty() {
        return Ok("No unused genres found. All genres in the dropdown are being used by books.".to_string());
    }
    
    // Step 4: Delete unused genres from AudiobookShelf
    let mut deleted_count = 0;
    let mut failed_count = 0;
    
    for genre in &unused_genres {
        let delete_url = format!("{}/api/me/item/{}", config.abs_base_url, urlencoding::encode(genre));
        let delete_result = client
            .delete(&delete_url)
            .header("Authorization", format!("Bearer {}", config.abs_api_token))
            .send()
            .await;
        
        match delete_result {
            Ok(resp) if resp.status().is_success() => deleted_count += 1,
            _ => failed_count += 1,
        }
    }
    
    Ok(format!(
        "Removed {} unused genres from dropdown. {} failed.\nRemoved: {}",
        deleted_count,
        failed_count,
        unused_genres.join(", ")
    ))
}

#[tauri::command]
async fn normalize_genres() -> Result<String, String> {
    let config = config::load_config().map_err(|e| e.to_string())?;
    
    if config.abs_base_url.is_empty() || config.abs_api_token.is_empty() || config.abs_library_id.is_empty() {
        return Err("AudiobookShelf not configured. Please set Base URL, API Token, and Library ID in Settings.".to_string());
    }
    
    let client = reqwest::Client::new();
    
    // Get all library items
    let url = format!("{}/api/libraries/{}/items?limit=1000", config.abs_base_url, config.abs_library_id);
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.abs_api_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch library items: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch library items: {}", response.status()));
    }
    
    let items: LibraryItemsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse library items: {}", e))?;
    
    let mut updated_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    
    // Normalize genres for each item
    for item in items.results {
        if let Some(current_genres) = &item.media.metadata.genres {
            if current_genres.is_empty() {
                skipped_count += 1;
                continue;
            }
            
            // Map genres to approved list
            let normalized_genres = genres::enforce_genre_policy_basic(current_genres);
            
            // Only update if genres actually changed
            if normalized_genres != *current_genres {
                let update_url = format!("{}/api/items/{}/media", config.abs_base_url, item.id);
                let update_result = client
                    .patch(&update_url)
                    .header("Authorization", format!("Bearer {}", config.abs_api_token))
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({
                        "metadata": {
                            "genres": normalized_genres
                        }
                    }))
                    .send()
                    .await;
                
                match update_result {
                    Ok(resp) if resp.status().is_success() => updated_count += 1,
                    _ => failed_count += 1,
                }
            } else {
                skipped_count += 1;
            }
        } else {
            skipped_count += 1;
        }
    }
    
    Ok(format!("Normalized {} items, skipped {} (already correct/empty), {} failed.", 
        updated_count, skipped_count, failed_count))
}

// ============================================================================
// AUDIBLE AUTH COMMANDS
// ============================================================================

#[tauri::command]
async fn login_to_audible(email: String, password: String, country_code: String) -> Result<String, String> {
    audible_auth::login_audible(&email, &password, &country_code)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_audible_installed() -> Result<bool, String> {
    audible_auth::check_audible_status()
        .map_err(|e| e.to_string())
}

// ============================================================================
// MAIN FUNCTION
// ============================================================================

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Core commands
            scan_library,
            write_tags,
            get_config,
            save_config,
            test_abs_connection,
            // Maintenance commands
            clear_cache,
            restart_abs_docker,
            force_abs_rescan,
            clear_abs_cache,
            clear_all_genres,
            normalize_genres,
            // Audible commands
            login_to_audible,
            check_audible_installed,
            inspect_file_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}