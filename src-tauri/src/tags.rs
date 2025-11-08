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
    
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", file_path);
    }
    
    if backup {
        let backup_path = path.with_extension(
            format!("{}.backup", path.extension().unwrap_or_default().to_string_lossy())
        );
        std::fs::copy(path, &backup_path)?;
        println!("📋 Backup created: {}", backup_path.display());
    }
    
    println!("📝 Writing tags to: {}", file_path);
    
    let mut tagged_file = Probe::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open file: {}", e))?
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to read file tags: {}", e))?;
    
    let tag = if let Some(t) = tagged_file.primary_tag_mut() {
        t
    } else {
        let tag_type = tagged_file.primary_tag_type();
        tagged_file.insert_tag(Tag::new(tag_type));
        tagged_file.primary_tag_mut().unwrap()
    };
    
    for (field, change) in changes {
        println!("   🔧 Updating {}: '{}' -> '{}'", field, change.old, change.new);
        
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
                tag.remove_key(&ItemKey::Genre);
                
                let genres: Vec<&str> = change.new
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                for genre in &genres {
                    let item = TagItem::new(
                        ItemKey::Genre,
                        ItemValue::Text(genre.to_string())
                    );
                    tag.push(item);
                }
                
                println!("   ✅ Wrote {} separate genre tags: {:?}", genres.len(), genres);
            },
            "narrator" => {
                tag.remove_key(&ItemKey::Composer);
                tag.insert_text(ItemKey::Composer, change.new.clone());
                tag.remove_key(&ItemKey::Comment);
                
                println!("   ✅ Wrote narrator to Composer: {}", change.new);
            },
            "description" | "comment" => {
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
            _ => {
                println!("   ⚠️  Unknown field type: {}", field);
            }
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    tagged_file.save_to_path(path, lofty::config::WriteOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to save tags: {}", e))?;
    
    println!("✅ Saved tags to: {}", file_path);
    
    Ok(())
}

pub fn verify_genres(file_path: &str) -> Result<Vec<String>> {
    let tagged_file = Probe::open(file_path)?.read()?;
    let tag = tagged_file.primary_tag().ok_or_else(|| anyhow::anyhow!("No tag found"))?;
    
    let genres: Vec<String> = tag
        .get_strings(&ItemKey::Genre)
        .map(|s| s.to_string())
        .collect();
    
    Ok(genres)
}
