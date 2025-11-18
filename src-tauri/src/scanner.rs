use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use anyhow::Result;

use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};

static CANCELLATION_FLAG: AtomicBool = AtomicBool::new(false);

pub fn set_cancellation_flag(cancelled: bool) {
    CANCELLATION_FLAG.store(cancelled, Ordering::Relaxed);
}

pub fn is_cancelled() -> bool {
    CANCELLATION_FLAG.load(Ordering::Relaxed)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFileData {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub tags: FileTags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub track: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookGroup {
    pub id: String,
    pub group_name: String,
    pub group_type: GroupType,
    pub files: Vec<AudioFile>,
    pub metadata: BookMetadata,
    pub total_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub status: String,
    pub changes: HashMap<String, FieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GroupType {
    Single,
    Chapters,
    Series,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub narrator: Option<String>,
    pub series: Option<String>,
    pub sequence: Option<String>,
    pub genres: Vec<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub description: Option<String>,
    pub isbn: Option<String>,
}

fn is_already_processed(tags: &FileTags) -> bool {
    // Check if tags match our app's output format
    let has_narrator_format = tags.comment.as_ref()
        .map(|c| c.contains("Narrated by ") || c.contains("Read by "))
        .unwrap_or(false);
    
    let has_clean_genres = tags.genre.as_ref()
        .map(|g| {
            // Check if it's our comma-separated format with approved genres
            let genre_parts: Vec<&str> = g.split(',').map(|s| s.trim()).collect();
            genre_parts.len() >= 1 && genre_parts.len() <= 3 && 
            genre_parts.iter().any(|&genre| crate::genres::APPROVED_GENRES.contains(&genre))
        })
        .unwrap_or(false);
    
    let has_clean_title = tags.title.as_ref()
        .map(|t| !t.contains("(Unabridged)") && !t.contains("[Retail]") && !t.contains("320kbps") && !t.contains("Track "))
        .unwrap_or(false);
    
    println!("🔍 Already processed check:");
    println!("   Narrator format: {} (comment: {:?})", has_narrator_format, tags.comment);
    println!("   Clean genres: {} (genre: {:?})", has_clean_genres, tags.genre);
    println!("   Clean title: {} (title: {:?})", has_clean_title, tags.title);
    
    // File is considered "already processed" if it has our narrator format AND clean genres
    let is_processed = has_narrator_format && has_clean_genres;
    println!("   RESULT: {}", if is_processed { "SKIP PROCESSING" } else { "NEEDS PROCESSING" });
    
    is_processed
}
// src-tauri/src/scanner.rs - Replace the scan_directory function
pub async fn scan_directory(
    dir_path: &str, 
    api_key: Option<String>,
    _skip_unchanged: bool,
    progress_callback: Option<Box<dyn Fn(crate::progress::ScanProgress) + Send + Sync>>
) -> Result<Vec<BookGroup>> {
    // CRITICAL: Reset cancellation flag at start
    set_cancellation_flag(false);
    
    println!("🔍 SCAN STARTED");
    println!("📂 Collecting files...");
    
    let files = collect_audio_files(dir_path)?;
    println!("📊 Found {} files\n", files.len());
    
    if files.is_empty() {
        return Ok(vec![]);
    }
    
    let groups = process_groups_with_gpt(files, api_key, _skip_unchanged, progress_callback).await;
    
    let total_changes: usize = groups.iter().map(|g| g.total_changes).sum();
    println!("✅ Complete: {} files in {} groups, {} changes", 
        groups.iter().map(|g| g.files.len()).sum::<usize>(),
        groups.len(),
        total_changes
    );
    
    Ok(groups)
}

fn collect_audio_files(dir_path: &str) -> Result<Vec<RawFileData>> {
    use walkdir::WalkDir;
    
    let mut files = Vec::new();
    
    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }
        
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        if !matches!(ext.as_str(), "m4b" | "m4a" | "mp3" | "flac" | "ogg") {
            continue;
        }
        
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        if filename.starts_with("._") || filename.starts_with(".DS_Store") {
            continue;
        }
        
        let tags = extract_tags(path);
        
        files.push(RawFileData {
            id: format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
            path: path.to_string_lossy().to_string(),
            filename,
            tags,
        });
    }
    
    Ok(files)
}

fn extract_tags(path: &Path) -> FileTags {
    use lofty::probe::Probe;
    use lofty::prelude::*;
    
    let tag = Probe::open(path)
        .ok()
        .and_then(|p| p.read().ok())
        .and_then(|t| t.primary_tag().cloned());
    
    FileTags {
        title: tag.as_ref().and_then(|t| t.title().map(|s| s.to_string())),
        artist: tag.as_ref().and_then(|t| t.artist().map(|s| s.to_string())),
        album: tag.as_ref().and_then(|t| t.album().map(|s| s.to_string())),
        album_artist: None,
        composer: None,
        genre: tag.as_ref().and_then(|t| t.genre().map(|s| s.to_string())),
        year: tag.as_ref().and_then(|t| t.year().map(|y| y.to_string())),
        track: None,
        comment: tag.as_ref().and_then(|t| t.comment().map(|s| s.to_string())),
    }
}

async fn process_groups_with_gpt(
    files: Vec<RawFileData>, 
    api_key: Option<String>,
    _skip_unchanged: bool,
    progress_callback: Option<Box<dyn Fn(crate::progress::ScanProgress) + Send + Sync>>
) -> Vec<BookGroup> {
    set_cancellation_flag(false);
    
    let total_files = files.len();
    let start_time = Instant::now();
    
    // ADD THIS LINE:
    crate::progress::set_total_files(total_files);
    
    let config = crate::config::load_config().ok();
    let max_workers = config.as_ref().map(|c| c.max_workers).unwrap_or(10);
    
    println!("🚀 Processing {} files with {} parallel workers...", total_files, max_workers);
    
    let mut folder_map: HashMap<String, Vec<RawFileData>> = HashMap::new();
    
    for file in files {
        let path = PathBuf::from(&file.path);
        let mut parent = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        parent = parent.replace("(book #", "(Book #").replace("(Book#", "(Book #");
        if !parent.ends_with(')') && parent.contains("Book #") {
            if let Some(pos) = parent.rfind(" - ") {
                parent = format!("{})", &parent[..pos]);
            }
        }
        if is_cancelled() {
            println!("🛑 Scan cancelled by user");
            break;
        }
        let _filename_lower = file.filename.to_lowercase();
        let parent_lower = parent.to_lowercase();
        
        let group_key = if parent_lower.contains("book #") || parent_lower.contains("book#") {
            if let Some(book_match) = parent_lower.split("book #").nth(1)
                .or_else(|| parent_lower.split("book#").nth(1)) {
                if let Some(book_num_end) = book_match.find(|c: char| !c.is_numeric() && c != ')') {
                    let book_id = &book_match[..book_num_end];
                    let base_parent = if let Some(pos) = parent.find("(Book #") {
                        parent[..pos].trim().to_string()
                    } else if let Some(pos) = parent.find("(book #") {
                        parent[..pos].trim().to_string()
                    } else {
                        parent.clone()
                    };
                    format!("{} (Book #{})", base_parent, book_id)
                } else {
                    parent.clone()
                }
            } else {
                parent.clone()
            }
        } else {
            parent.clone()
        };
        
        folder_map.entry(group_key).or_insert_with(Vec::new).push(file);
    }
    
    let mut groups = Vec::new();
    let mut group_id = 0;
    let total_groups = folder_map.len();
    let mut progress = crate::progress::ScanProgress::new(total_groups);
    let mut processed = 0;
    
    for (folder_name, mut folder_files) in folder_map {
        if is_cancelled() {
            println!("🛑 Scan cancelled by user");
            break;
        }
        crate::progress::increment_progress(&folder_name);
        
        println!("📁 Processing group: {} [{}]", folder_name, folder_files[0].id);
        // ... rest of the existing code
        processed += 1;
        progress.update(processed, &folder_name, start_time, false);
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }
        
        folder_files.sort_by(|a, b| a.filename.cmp(&b.filename));
        
        let group_type = detect_group_type(&folder_files);
        
        println!("\n📁 Processing group: {}", folder_name);
        println!("   Type: {:?}, Files: {}", group_type, folder_files.len());
        
        if matches!(group_type, GroupType::Series) && folder_files.len() > 1 {
            println!("   📚 Series detected - processing each book separately");
            
            for file in folder_files {
                 if is_cancelled() {
                    println!("🛑 Scan cancelled by user - stopping series processing");
                    break;
                }
                let book_name = file.filename.replace(".m4b", "").replace(".m4a", "").replace(".mp3", "");
                
                println!("\n   📖 Book: {}", book_name);
                println!("      🔍 Step 1: GPT extracts book info...");
                let (book_title, book_author) = extract_book_info_with_gpt(
                    &file,
                    &book_name,
                    api_key.as_deref()
                ).await;
                
                println!("      ✅ Extracted: title='{}', author='{}'", book_title, book_author);
                
                let config = crate::config::load_config().ok();
                
                println!("      🎧 Step 2: Query Audible (Primary)...");
                let audible_data = if let Some(ref cfg) = config {
                    if cfg.audible_enabled && !cfg.audible_cli_path.is_empty() {
                        crate::audible::search_audible(&book_title, &book_author, &cfg.audible_cli_path)
                            .await.ok().flatten()
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                println!("      📚 Step 3: Query Google Books (Fallback)...");
                let google_data = crate::metadata::fetch_from_google_books(&book_title, &book_author)
                    .await.ok().flatten();
                
                println!("      🤖 Step 4: GPT merges all sources...");
                let final_metadata = merge_all_with_gpt_retry(
                    &[file.clone()],
                    &book_name,
                    &book_title,
                    &book_author,
                    google_data,
                    audible_data,
                    api_key.as_deref(),
                    3
                ).await;
                
                let mut changes = HashMap::new();
                
                if let Some(old_title) = &file.tags.title {
                    if old_title != &final_metadata.title {
                        changes.insert("title".to_string(), FieldChange {
                            old: old_title.clone(),
                            new: final_metadata.title.clone(),
                        });
                    }
                }
                
                if let Some(old_artist) = &file.tags.artist {
                    if old_artist != &final_metadata.author {
                        changes.insert("author".to_string(), FieldChange {
                            old: old_artist.clone(),
                            new: final_metadata.author.clone(),
                        });
                    }
                }
                
                if let Some(narrator) = &final_metadata.narrator {
                    changes.insert("narrator".to_string(), FieldChange {
                        old: file.tags.comment.clone().unwrap_or_default(),
                        new: format!("Narrated by {}", narrator),
                    });
                }
                
                if !final_metadata.genres.is_empty() {
                    let new_genre = final_metadata.genres.join(", ");
                    if let Some(old_genre) = &file.tags.genre {
                        if old_genre != &new_genre {
                            changes.insert("genre".to_string(), FieldChange {
                                old: old_genre.clone(),
                                new: new_genre,
                            });
                        }
                    } else {
                        changes.insert("genre".to_string(), FieldChange {
                            old: String::new(),
                            new: new_genre,
                        });
                    }
                }
                
                let audio_file = AudioFile {
                    id: file.id.clone(),
                    path: file.path.clone(),
                    filename: file.filename.clone(),
                    status: if changes.is_empty() { "unchanged" } else { "changed" }.to_string(),
                    changes,
                };
                
                let total_changes = if audio_file.changes.is_empty() { 0 } else { 1 };
                
                groups.push(BookGroup {
                    id: group_id.to_string(),
                    group_name: book_name,
                    group_type: GroupType::Single,
                    files: vec![audio_file],
                    metadata: final_metadata,
                    total_changes,
                });
                
                group_id += 1;
            }
            
            continue;
        }
        
        let sample_file = &folder_files[0];
        // OPTIMIZATION: Try cache FIRST before any GPT calls
        let cache = crate::cache::MetadataCache::new().ok();
        let config = crate::config::load_config().ok();
        
        // Quick check: try to extract title/author from filename for cache lookup
        let quick_title = sample_file.tags.title.as_deref()
            .unwrap_or(&folder_name);
        let quick_author = sample_file.tags.artist.as_deref()
            .or(sample_file.tags.album_artist.as_deref())
            .unwrap_or("Unknown");
        
        // NEW: Check if file was already processed by our app
        let already_processed = is_already_processed(&sample_file.tags);
        if already_processed {
            println!("   ✅ File already processed by this app - using existing tags directly");
            println!("   📋 Title: {:?}", sample_file.tags.title);
            println!("   📋 Comment: {:?}", sample_file.tags.comment);
            println!("   📋 Genre: {:?}", sample_file.tags.genre);
        }
        
        if already_processed {
            println!("   ✅ File already processed by this app - using existing tags directly");
            
            // Extract existing tags directly without GPT reprocessing
            let final_metadata = BookMetadata {
                title: sample_file.tags.title.clone().unwrap_or_else(|| folder_name.clone()),
                subtitle: None,
                author: sample_file.tags.artist.clone().unwrap_or_else(|| "Unknown".to_string()),
                narrator: sample_file.tags.comment.as_ref()
                    .and_then(|c| {
                        if c.starts_with("Narrated by ") {
                            Some(c.trim_start_matches("Narrated by ").to_string())
                        } else if c.starts_with("Read by ") {
                            Some(c.trim_start_matches("Read by ").to_string())
                        } else {
                            None
                        }
                    }),
                series: None,
                sequence: None,
                genres: sample_file.tags.genre.as_ref()
                    .map(|g| g.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default(),
                publisher: None,
                year: sample_file.tags.year.clone(),
                description: None,
                isbn: None,
            };
            
            let audio_files: Vec<AudioFile> = folder_files.iter().map(|f| {
                AudioFile {
                    id: f.id.clone(),
                    path: f.path.clone(),
                    filename: f.filename.clone(),
                    status: "unchanged".to_string(),
                    changes: HashMap::new(),
                }
            }).collect();
            
            groups.push(BookGroup {
                id: group_id.to_string(),
                group_name: folder_name.clone(),
                group_type,
                files: audio_files,
                metadata: final_metadata,
                total_changes: 0,
            });
            
            group_id += 1;
            continue;
        }
        
        // Check cache with quick lookup first
        if let Some(ref cache_db) = cache {
            if let Some(cached) = cache_db.get(quick_title, quick_author) {
                println!("   💾 Using cached metadata (FAST PATH - skipping ALL GPT calls)");
                
                let final_metadata = cached.final_metadata;
                
                let audio_files: Vec<AudioFile> = folder_files.iter().map(|f| {
                    let mut changes = HashMap::new();
                    
                    if let Some(old_title) = &f.tags.title {
                        if old_title != &final_metadata.title {
                            changes.insert("title".to_string(), FieldChange {
                                old: old_title.clone(),
                                new: final_metadata.title.clone(),
                            });
                        }
                    }
                    
                    if let Some(old_artist) = &f.tags.artist {
                        if old_artist != &final_metadata.author {
                            changes.insert("author".to_string(), FieldChange {
                                old: old_artist.clone(),
                                new: final_metadata.author.clone(),
                            });
                        }
                    }
                    
                    if let Some(narrator) = &final_metadata.narrator {
                        changes.insert("narrator".to_string(), FieldChange {
                            old: f.tags.comment.clone().unwrap_or_default(),
                            new: format!("Narrated by {}", narrator),
                        });
                    }
                    
                    if !final_metadata.genres.is_empty() {
                        let new_genre = final_metadata.genres.join(", ");
                        if let Some(old_genre) = &f.tags.genre {
                            if old_genre != &new_genre {
                                changes.insert("genre".to_string(), FieldChange {
                                    old: old_genre.clone(),
                                    new: new_genre,
                                });
                            }
                        } else {
                            changes.insert("genre".to_string(), FieldChange {
                                old: String::new(),
                                new: new_genre,
                            });
                        }
                    }
                    
                    AudioFile {
                        id: f.id.clone(),
                        path: f.path.clone(),
                        filename: f.filename.clone(),
                        status: if changes.is_empty() { "unchanged" } else { "changed" }.to_string(),
                        changes,
                    }
                }).collect();
                
                let total_changes = audio_files.iter().filter(|f| !f.changes.is_empty()).count();
                
                groups.push(BookGroup {
                    id: group_id.to_string(),
                    group_name: folder_name.clone(),
                    group_type,
                    files: audio_files,
                    metadata: final_metadata,
                    total_changes,
                });
                
                group_id += 1;
                continue;
            }
        }
        
        // CACHE MISS - Need to do full processing
        println!("   🔍 Step 1: GPT extracts book info from tags...");
        let (book_title, book_author) = extract_book_info_with_gpt(
            sample_file,
            &folder_name,
            api_key.as_deref()
        ).await;
        
        println!("   ✅ Extracted: title='{}', author='{}'", book_title, book_author);
        
        let cache = crate::cache::MetadataCache::new().ok();
        
        let (audible_data, google_data) = if let Some(ref cache_db) = cache {
            // This shouldn't happen since we checked cache above, but keeping for safety
            if let Some(_cached) = cache_db.get(&book_title, &book_author) {
                println!("   💾 Using cached metadata");
                // This case is now handled above, but keeping fallback
                (None, None)
            } else {
                println!("   🎧 Step 2: Query Audible (Primary)...");
                let audible = if let Some(ref cfg) = config {
                    if cfg.audible_enabled && !cfg.audible_cli_path.is_empty() {
                        crate::audible::search_audible(&book_title, &book_author, &cfg.audible_cli_path)
                            .await.ok().flatten()
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                println!("   📚 Step 3: Query Google Books (Fallback)...");
                let google = crate::metadata::fetch_from_google_books(&book_title, &book_author)
                    .await.ok().flatten();
                
                (audible, google)
            }
        } else {
            println!("   🎧 Step 2: Query Audible (Primary)...");
            let audible = if let Some(ref cfg) = config {
                if cfg.audible_enabled && !cfg.audible_cli_path.is_empty() {
                    crate::audible::search_audible(&book_title, &book_author, &cfg.audible_cli_path)
                        .await.ok().flatten()
                } else {
                    None
                }
            } else {
                None
            };
            
            println!("   📚 Step 3: Query Google Books (Fallback)...");
            let google = crate::metadata::fetch_from_google_books(&book_title, &book_author)
                .await.ok().flatten();
            
            (audible, google)
        };
        
        println!("   🤖 Step 4: GPT merges all sources...");
        let final_metadata = merge_all_with_gpt_retry(
            &folder_files,
            &folder_name,
            &book_title,
            &book_author,
            google_data,
            audible_data,
            api_key.as_deref(),
            3
        ).await;
        
        // Store FINAL metadata in cache for next time
        if let Some(ref cache_db) = cache {
            let _ = cache_db.set(&book_title, &book_author, crate::cache::CachedMetadata {
                final_metadata: final_metadata.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        let audio_files: Vec<AudioFile> = folder_files.iter().map(|f| {
            let mut changes = HashMap::new();
            
            if let Some(old_title) = &f.tags.title {
                if old_title != &final_metadata.title {
                    changes.insert("title".to_string(), FieldChange {
                        old: old_title.clone(),
                        new: final_metadata.title.clone(),
                    });
                }
            }
            
            if let Some(old_artist) = &f.tags.artist {
                if old_artist != &final_metadata.author {
                    changes.insert("author".to_string(), FieldChange {
                        old: old_artist.clone(),
                        new: final_metadata.author.clone(),
                    });
                }
            }
            
            if let Some(narrator) = &final_metadata.narrator {
                changes.insert("narrator".to_string(), FieldChange {
                    old: f.tags.comment.clone().unwrap_or_default(),
                    new: format!("Narrated by {}", narrator),
                });
            }
            
            if !final_metadata.genres.is_empty() {
                let new_genre = final_metadata.genres.join(", ");
                if let Some(old_genre) = &f.tags.genre {
                    if old_genre != &new_genre {
                        changes.insert("genre".to_string(), FieldChange {
                            old: old_genre.clone(),
                            new: new_genre,
                        });
                    }
                } else {
                    changes.insert("genre".to_string(), FieldChange {
                        old: String::new(),
                        new: new_genre,
                    });
                }
            }
            
            AudioFile {
                id: f.id.clone(),
                path: f.path.clone(),
                filename: f.filename.clone(),
                status: if changes.is_empty() { "unchanged" } else { "changed" }.to_string(),
                changes,
            }
        }).collect();
        
        let total_changes = audio_files.iter().filter(|f| !f.changes.is_empty()).count();
        
        groups.push(BookGroup {
            id: group_id.to_string(),
            group_name: folder_name,
            group_type,
            files: audio_files,
            metadata: final_metadata,
            total_changes,
        });
        
        group_id += 1;
    }
    
    groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    
    let elapsed = start_time.elapsed();
    let rate = total_files as f64 / elapsed.as_secs_f64();
    println!("\n⚡ Performance: {:.1} files/sec, total time: {:?}", rate, elapsed);
    
    groups
}
async fn extract_book_info_with_gpt(
    sample_file: &RawFileData,
    folder_name: &str,
    api_key: Option<&str>
) -> (String, String) {
    let api_key = match api_key {
        Some(key) if !key.is_empty() => key,
        _ => {
            return (
                sample_file.tags.title.clone().unwrap_or_else(|| folder_name.to_string()),
                sample_file.tags.artist.clone().unwrap_or_else(|| String::from("Unknown"))
            );
        }
    };
    
    // Clean our own formatting before sending to GPT
    let clean_title = sample_file.tags.title.as_ref()
        .map(|t| t.replace(" - Part 1", "").replace(" - Part 2", "").trim().to_string());
    let clean_artist = sample_file.tags.artist.as_ref()
        .map(|a| a.to_string());
    
    let prompt = format!(
r#"Extract the BOOK title and AUTHOR from these audiobook file tags. Ignore chapter/track numbers.

FOLDER NAME: {}
FILENAME: {}
FILE TAGS:
- Title: {:?}
- Artist: {:?}
- Album: {:?}

IMPORTANT: These tags may have been cleaned already. Use them directly if they look clean.
- If title is already clean (no track numbers, no junk), use it as-is
- If artist is already clean, use it as-is

The tags might be messy (e.g., "Track 10" or "Magic Tree House - #55 Night of the Ninth Dragon").

Extract the actual BOOK title and AUTHOR name. Remove:
- Track/Chapter numbers
- Book numbers (#54, #55, etc)
- Series markers
- File format info (320kbps, Unabridged, etc)

Return ONLY valid JSON:
{{"book_title":"actual book title","author":"author name"}}

JSON:"#,
        folder_name,
        sample_file.filename,
        clean_title,
        clean_artist,
        sample_file.tags.album
    );
    
    match call_gpt_extract_book_info(&prompt, api_key).await {
        Ok(json_str) => {
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(json) => {
                    let title = json["book_title"].as_str()
                        .unwrap_or(&sample_file.tags.title.as_deref().unwrap_or(folder_name))
                        .to_string();
                    let author = json["author"].as_str()
                        .unwrap_or(&sample_file.tags.artist.as_deref().unwrap_or("Unknown"))
                        .to_string();
                    (title, author)
                }
                Err(_) => {
                    (
                        sample_file.tags.title.clone().unwrap_or_else(|| folder_name.to_string()),
                        sample_file.tags.artist.clone().unwrap_or_else(|| String::from("Unknown"))
                    )
                }
            }
        }
        Err(e) => {
            println!("   ⚠️  GPT extraction error: {}", e);
            (
                sample_file.tags.title.clone().unwrap_or_else(|| folder_name.to_string()),
                sample_file.tags.artist.clone().unwrap_or_else(|| String::from("Unknown"))
            )
        }
    }
}
async fn merge_all_with_gpt(
    files: &[RawFileData],
    folder_name: &str,
    extracted_title: &str,
    extracted_author: &str,
    google_data: Option<crate::metadata::BookMetadata>,
    audible_data: Option<crate::audible::AudibleMetadata>,
    api_key: Option<&str>
) -> BookMetadata {
    let sample_comments: Vec<String> = files.iter()
        .filter_map(|f| f.tags.comment.clone())
        .collect();
    
    // PRE-EXTRACT reliable year from sources (don't let GPT override this)
    let reliable_year = audible_data.as_ref()
        .and_then(|d| d.release_date.clone())
        .and_then(|date| {
            // Extract just the year from date strings like "2021-01-02"
            date.split('-').next().map(|s| s.to_string())
        })
        .or_else(|| {
            google_data.as_ref()
                .and_then(|d| d.publish_date.clone())
                .and_then(|date| {
                    date.split('-').next().map(|s| s.to_string())
                })
        });
    
    let google_summary = if let Some(ref data) = google_data {
        format!(
            "Title: {:?}, Authors: {:?}, Publisher: {:?}, Date: {:?}",
            data.title, data.authors, data.publisher, data.publish_date
        )
    } else {
        "No data".to_string()
    };
    
    let audible_summary = if let Some(ref data) = audible_data {
        format!(
            "Title: {:?}, Authors: {:?}, Narrators: {:?}, Series: {:?}, Publisher: {:?}, Release Date: {:?}, ASIN: {:?}",
            data.title, data.authors, data.narrators, data.series, data.publisher, data.release_date, data.asin
        )
    } else {
        "No data".to_string()
    };
    
    let api_key = match api_key {
        Some(key) if !key.is_empty() => key,
        _ => {
            return BookMetadata {
                title: extracted_title.to_string(),
                subtitle: None,
                author: extracted_author.to_string(),
                narrator: None,
                series: None,
                sequence: None,
                genres: vec![],
                publisher: google_data.as_ref().and_then(|d| d.publisher.clone()),
                year: reliable_year,
                description: google_data.as_ref().and_then(|d| d.description.clone()),
                isbn: None,
            };
        }
    };
    
    let year_instruction = if let Some(ref year) = reliable_year {
        format!("CRITICAL: Use EXACTLY this year: {} (from Audible/Google Books - DO NOT CHANGE)", year)
    } else {
        "year: If not found in sources, return null".to_string()
    };
    
    let prompt = format!(
r#"You are an audiobook metadata expert. Merge data from multiple sources into the best metadata.

SOURCES:
1. Folder: {}
2. Extracted from tags: title='{}', author='{}'
3. Google Books: {}
4. Audible: {}
5. Sample comments: {:?}
6. FILENAME HINT: Look at folder/filename for series info

INSTRUCTIONS FOR SERIES:
If folder has patterns like Book 01 or War of The Roses 01, extract series and sequence.

APPROVED GENRES (max 3, comma-separated):
{}

OUTPUT ALL FIELDS:
- title: Book title (not chapter). Remove junk.
- subtitle: If available from Google Books or Audible
- author: Clean author name
- narrator: Extract from Audible narrators field or look for Narrated by in comments
- series: Extract from filename pattern if book number present
- sequence: Book number
- sequence: Extract book number from filename (e.g., "01" or "02")
- genres: Pick 1-3 from approved list
- publisher: From Google Books or Audible
- {}
- description: Brief description from Google/Audible
- isbn: From Google Books

Return ONLY valid JSON:
{{"title":"...","subtitle":null,"author":"...","narrator":"...","series":"...","sequence":"...","genres":["..."],"publisher":"...","year":"...","description":"...","isbn":"..."}}

JSON:"#,
        folder_name,
        extracted_title,
        extracted_author,
        google_summary,
        audible_summary,
        sample_comments,
        crate::genres::APPROVED_GENRES.join(", "),
        year_instruction
    );
    
    match call_gpt_merge_metadata(&prompt, api_key).await {
        Ok(json_str) => {
            match serde_json::from_str::<BookMetadata>(&json_str) {
                Ok(mut metadata) => {
                    // FORCE the reliable year back in (in case GPT changed it)
                    if let Some(year) = reliable_year {
                        metadata.year = Some(year);
                    }
                    
                    println!("   ✅ Final: title='{}', author='{}', narrator={:?}", 
                        metadata.title, metadata.author, metadata.narrator);
                    println!("            genres={:?}, publisher={:?}, year={:?}",
                        metadata.genres, metadata.publisher, metadata.year);
                    metadata
                }
                Err(e) => {
                    println!("   ⚠️  GPT parse error: {}", e);
                    println!("   ⚠️  Using fallback with available data");
                    
                    BookMetadata {
                        title: extracted_title.to_string(),
                        subtitle: google_data.as_ref().and_then(|d| d.subtitle.clone()),
                        author: extracted_author.to_string(),
                        narrator: audible_data.as_ref()
                            .and_then(|d| d.narrators.first().cloned()),
                        series: audible_data.as_ref()
                            .and_then(|d| d.series.first().map(|s| s.name.clone())),
                        sequence: audible_data.as_ref()
                            .and_then(|d| d.series.first().and_then(|s| s.position.clone())),
                        genres: google_data.as_ref()
                            .map(|d| d.genres.clone())
                            .unwrap_or_default(),
                        publisher: google_data.as_ref().and_then(|d| d.publisher.clone())
                            .or_else(|| audible_data.as_ref().and_then(|d| d.publisher.clone())),
                        year: reliable_year,
                        description: google_data.as_ref().and_then(|d| d.description.clone())
                            .or_else(|| audible_data.as_ref().and_then(|d| d.description.clone())),
                        isbn: google_data.as_ref()
                            .and_then(|d| d.isbn.clone()),
                    }
                }
            }
        }
        Err(e) => {
            println!("   ⚠️  GPT merge error: {}", e);
            println!("   ⚠️  Using fallback with available data");
            
            BookMetadata {
                title: extracted_title.to_string(),
                subtitle: google_data.as_ref().and_then(|d| d.subtitle.clone()),
                author: extracted_author.to_string(),
                narrator: audible_data.as_ref()
                    .and_then(|d| d.narrators.first().cloned()),
                series: audible_data.as_ref()
                    .and_then(|d| d.series.first().map(|s| s.name.clone())),
                sequence: audible_data.as_ref()
                    .and_then(|d| d.series.first().and_then(|s| s.position.clone())),
                genres: google_data.as_ref()
                    .map(|d| d.genres.clone())
                    .unwrap_or_default(),
                publisher: google_data.as_ref().and_then(|d| d.publisher.clone())
                    .or_else(|| audible_data.as_ref().and_then(|d| d.publisher.clone())),
                year: reliable_year,
                description: google_data.as_ref().and_then(|d| d.description.clone())
                    .or_else(|| audible_data.as_ref().and_then(|d| d.description.clone())),
                isbn: google_data.as_ref()
                    .and_then(|d| d.isbn.clone()),
            }
        }
    }
}

async fn call_gpt_extract_book_info(prompt: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-5-nano",
            "messages": [
                {
                    "role": "system",
                    "content": "Extract book info. Return JSON: {\"book_title\":\"...\",\"author\":\"...\"}"
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_completion_tokens": 300,
            "verbosity": "low",
            "reasoning_effort": "minimal"
        }))
        .send()
        .await?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    if !status.is_success() {
        println!("             ❌ API Error ({}): {}", status, response_text);
        anyhow::bail!("API returned status {}: {}", status, response_text);
    }
    
    parse_gpt_response(&response_text)
}

async fn call_gpt_merge_metadata(prompt: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-5-nano",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an audiobook metadata expert. Return valid JSON only."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_completion_tokens": 4000,
            "verbosity": "low",
            "reasoning_effort": "minimal"
        }))
        .send()
        .await?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    if !status.is_success() {
        println!("             ❌ API Error ({}): {}", status, response_text);
        anyhow::bail!("API returned status {}: {}", status, response_text);
    }
    
    parse_gpt_response(&response_text)
}

fn parse_gpt_response(response_text: &str) -> Result<String> {
    println!("             🔍 DEBUG: Raw API response (first 500 chars): {}", &response_text[..response_text.len().min(500)]);
    
    #[derive(serde::Deserialize)]
    struct Response {
        choices: Vec<Choice>,
    }
    
    #[derive(serde::Deserialize)]
    struct Choice {
        message: Message,
    }
    
    #[derive(serde::Deserialize)]
    struct Message {
        content: String,
    }
    
    let result: Response = serde_json::from_str(response_text)?;
    
    println!("             🔍 DEBUG: Number of choices: {}", result.choices.len());
    
    let content = result.choices.first()
        .ok_or_else(|| anyhow::anyhow!("No choices"))?
        .message.content.trim();
    
    println!("             🔍 DEBUG: Content length: {}, Content preview: {}", content.len(), &content[..content.len().min(100)]);
    
    if content.is_empty() {
        anyhow::bail!("GPT returned empty content");
    }
    
    let json_str = content
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    
    println!("             🔍 DEBUG: Final JSON (first 200 chars): {}", &json_str[..json_str.len().min(200)]);
    
    Ok(json_str.to_string())
}

fn detect_group_type(files: &[RawFileData]) -> GroupType {
    if files.len() == 1 {
        return GroupType::Single;
    }
    
    let filenames: Vec<String> = files.iter().map(|f| f.filename.to_lowercase()).collect();
    
    let has_chapter_indicators = filenames.iter().any(|name| {
        name.contains(" ch ") || name.contains(" ch.") ||
        name.contains("chapter") || 
        name.contains("track") ||
        name.contains("part ") ||
        name.contains("disc") ||
        name.starts_with("01 ") || name.starts_with("02 ") || name.starts_with("03 ") ||
        name.starts_with("1 ") || name.starts_with("2 ") || name.starts_with("3 ") ||
        name.starts_with("01-") || name.starts_with("02-") || name.starts_with("03-") ||
        name.starts_with("001 ") || name.starts_with("002 ") || name.starts_with("003 ")
    });
    
    let all_same_title = files.iter()
        .filter_map(|f| f.tags.title.as_ref())
        .collect::<std::collections::HashSet<_>>()
        .len() == 1;
    
    if has_chapter_indicators || all_same_title {
        return GroupType::Chapters;
    }
    
    if files.len() > 5 {
        return GroupType::Chapters;
    }
    
    GroupType::Chapters
}
// ============================================================================
// RETRY LOGIC WITH QUALITY VALIDATION
// ============================================================================

async fn merge_all_with_gpt_retry(
    files: &[RawFileData],
    folder_name: &str,
    extracted_title: &str,
    extracted_author: &str,
    google_data: Option<crate::metadata::BookMetadata>,
    audible_data: Option<crate::audible::AudibleMetadata>,
    api_key: Option<&str>,
    max_retries: u32,
) -> BookMetadata {
    for attempt in 1..=max_retries {
        if attempt > 1 {
            println!("   🔄 Retry attempt {}/{}", attempt, max_retries);
        }
        
        let metadata = merge_all_with_gpt(
            files,
            folder_name,
            extracted_title,
            extracted_author,
            google_data.clone(),
            audible_data.clone(),
            api_key
        ).await;
        
        let quality_score = validate_metadata_quality(&metadata, extracted_title, &audible_data);
        
        if quality_score >= 80 {
            println!("   ✅ Quality: {}% - PASSED", quality_score);
            return metadata;
        } else {
            println!("   ⚠️  Quality: {}% - RETRY", quality_score);
        }
    }
    
    println!("   ⚠️  All retries exhausted, using last result");
    merge_all_with_gpt(files, folder_name, extracted_title, extracted_author, google_data, audible_data, api_key).await
}

fn validate_metadata_quality(
    metadata: &BookMetadata,
    extracted_title: &str,
    audible_data: &Option<crate::audible::AudibleMetadata>,
) -> u32 {
    let mut score = 0;
    
    // Title must include the extracted title (e.g., "Dinosaurs Before Dark")
    if metadata.title.contains(extracted_title) {
        score += 30;
    } else {
        println!("      ❌ Title doesn't contain '{}'", extracted_title);
    }
    
    // Narrator must exist if Audible has it
    if let Some(aud) = audible_data {
        if !aud.narrators.is_empty() {
            if metadata.narrator.is_some() {
                score += 20;
            } else {
                println!("      ❌ Missing narrator (Audible has: {:?})", aud.narrators);
            }
        }
    }
    
    // Description should exist and be substantial
    if let Some(ref desc) = metadata.description {
        if desc.len() >= 100 && desc.len() <= 1000 {
            score += 20;
        }
    }
    
    // Genres should be valid
    if !metadata.genres.is_empty() && metadata.genres.len() <= 3 {
        score += 15;
    }
    
    // Series/sequence should match if present
    if metadata.series.is_some() && metadata.sequence.is_some() {
        score += 10;
    }
    
    // Has publication info
    if metadata.publisher.is_some() || metadata.year.is_some() {
        score += 5;
    }
    
    score
}
