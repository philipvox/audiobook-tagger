use std::path::Path;
use anyhow::Result;
use lofty::probe::Probe;
use lofty::file::{TaggedFileExt, AudioFile};
use lofty::tag::{Accessor, Tag, ItemKey, ItemValue, TagItem};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteResult {
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<WriteError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteError {
    pub file_id: String,
    pub path: String,
    pub error: String,
}

pub async fn write_file_tags(
    file_path: &str,
    changes: &std::collections::HashMap<String, crate::scanner::FieldChange>,
    backup: bool,
) -> Result<()> {
    let path = Path::new(file_path);
    
    if backup {
        let backup_path = path.with_extension(
            format!("{}.backup", path.extension().unwrap_or_default().to_string_lossy())
        );
        std::fs::copy(path, backup_path)?;
    }
    
    let mut tagged_file = Probe::open(path)?.read()?;
    let tag = if let Some(t) = tagged_file.primary_tag_mut() {
        t
    } else {
        let tag_type = tagged_file.primary_tag_type();
        tagged_file.insert_tag(Tag::new(tag_type));
        tagged_file.primary_tag_mut().unwrap()
    };
    
    for (field, change) in changes {
        match field.as_str() {
            "title" => {
                tag.remove_key(&ItemKey::TrackTitle);
                tag.set_title(change.new.clone());
            },
            "artist" | "author" => {
                tag.remove_key(&ItemKey::TrackArtist);
                tag.set_artist(change.new.clone());
            },
            "album" => {
                tag.remove_key(&ItemKey::AlbumTitle);
                tag.set_album(change.new.clone());
            },
            "genre" => {
                // Remove all old genre tags first
                tag.remove_key(&ItemKey::Genre);
                
                // Split comma-separated genres
                let genres: Vec<&str> = change.new
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                // Write each genre as a separate TagItem
                for genre in &genres {
                    let item = TagItem::new(
                        ItemKey::Genre,
                        ItemValue::Text(genre.to_string())
                    );
                    tag.push(item);
                }
                
                println!("✅ Wrote {} separate genre tags: {:?}", genres.len(), genres);
            },
            "narrator" => {
                // CRITICAL: Only write to Composer field
                // AudiobookShelf reads narrator from Composer
                tag.remove_key(&ItemKey::Composer);
                tag.insert_text(ItemKey::Composer, change.new.clone());
                
                // IMPORTANT: Clear comment field to prevent narrator showing in description
                tag.remove_key(&ItemKey::Comment);
                
                println!("✅ Wrote narrator to Composer: {}", change.new);
            },
            "description" | "comment" => {
                // Only write actual description/comments (not narrator info)
                // Skip if it contains "narrated by" (case insensitive)
                if !change.new.to_lowercase().contains("narrated by") {
                    tag.set_comment(change.new.clone());
                }
            },
            "year" => {
                if let Ok(year) = change.new.parse::<u32>() {
                    tag.set_year(year);
                }
            },
            "series" => {
                tag.insert_text(ItemKey::Unknown("SERIES".to_string()), change.new.clone());
                tag.insert_text(ItemKey::Unknown("series".to_string()), change.new.clone());
            },
            "sequence" => {
                tag.insert_text(ItemKey::Unknown("SERIES-PART".to_string()), change.new.clone());
                tag.insert_text(ItemKey::Unknown("series-part".to_string()), change.new.clone());
            },
            _ => {}
        }
    }
    
    // Save with default options
    tagged_file.save_to_path(path, lofty::config::WriteOptions::default())?;
    
    println!("✅ Saved tags to: {}", file_path);
    
    Ok(())
}

// Helper function to verify genres were written correctly
pub fn verify_genres(file_path: &str) -> Result<Vec<String>> {
    let tagged_file = Probe::open(file_path)?.read()?;
    let tag = tagged_file.primary_tag().ok_or_else(|| anyhow::anyhow!("No tag found"))?;
    
    // Get all genre tags
    let genres: Vec<String> = tag
        .get_strings(&ItemKey::Genre)
        .map(|s| s.to_string())
        .collect();
    
    Ok(genres)
}