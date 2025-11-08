use std::path::Path;
use anyhow::Result;
use lofty::probe::Probe;
use lofty::file::AudioFile;
use lofty::tag::{Accessor, ItemKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RawTags {
    pub file_path: String,
    pub file_format: String,
    pub duration_seconds: Option<u64>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub tags: Vec<TagEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagEntry {
    pub key: String,
    pub value: String,
    pub tag_type: String,
}

pub fn inspect_file_tags(file_path: &str) -> Result<RawTags> {
    let path = Path::new(file_path);
    
    let tagged_file = Probe::open(path)?.read()?;
    
    // Get file format
    let file_format = format!("{:?}", tagged_file.file_type());
    
    // Get audio properties
    let properties = tagged_file.properties();
    let duration_seconds = Some(properties.duration().as_secs());
    let bitrate = properties.audio_bitrate();
    let sample_rate = properties.sample_rate();
    
    let mut tags = Vec::new();
    
    // Get all tags from the file
    if let Some(tag) = tagged_file.primary_tag() {
        let tag_type = format!("{:?}", tag.tag_type());
        
        // Standard fields
        if let Some(title) = tag.title() {
            tags.push(TagEntry {
                key: "Title".to_string(),
                value: title.to_string(),
                tag_type: tag_type.clone(),
            });
        }
        
        if let Some(artist) = tag.artist() {
            tags.push(TagEntry {
                key: "Artist/Author".to_string(),
                value: artist.to_string(),
                tag_type: tag_type.clone(),
            });
        }
        
        if let Some(album) = tag.album() {
            tags.push(TagEntry {
                key: "Album".to_string(),
                value: album.to_string(),
                tag_type: tag_type.clone(),
            });
        }
        
        if let Some(year) = tag.year() {
            tags.push(TagEntry {
                key: "Year".to_string(),
                value: year.to_string(),
                tag_type: tag_type.clone(),
            });
        }
        
        if let Some(comment) = tag.comment() {
            tags.push(TagEntry {
                key: "Comment".to_string(),
                value: comment.to_string(),
                tag_type: tag_type.clone(),
            });
        }
        
        // Get ALL genre tags (this will show if they're separated or not)
        let genres: Vec<String> = tag
            .get_strings(&ItemKey::Genre)
            .map(|s| s.to_string())
            .collect();
        
        if !genres.is_empty() {
            for (idx, genre) in genres.iter().enumerate() {
                tags.push(TagEntry {
                    key: format!("Genre #{}", idx + 1),
                    value: genre.clone(),
                    tag_type: tag_type.clone(),
                });
            }
        }
        
        // Composer (where narrator might be)
        let composers: Vec<String> = tag
            .get_strings(&ItemKey::Composer)
            .map(|s| s.to_string())
            .collect();
        
        if !composers.is_empty() {
            for composer in composers {
                tags.push(TagEntry {
                    key: "Composer (Narrator?)".to_string(),
                    value: composer,
                    tag_type: tag_type.clone(),
                });
            }
        }
        
        // Get ALL items (including custom tags)
        for item in tag.items() {
            let key_str = match item.key() {
                ItemKey::TrackTitle => "TrackTitle (Raw)".to_string(),
                ItemKey::TrackArtist => "TrackArtist (Raw)".to_string(),
                ItemKey::AlbumTitle => "AlbumTitle (Raw)".to_string(),
                ItemKey::Genre => continue, // Already handled above
                ItemKey::Comment => continue, // Already handled above
                ItemKey::Year => continue, // Already handled above
                ItemKey::Composer => continue, // Already handled above
                ItemKey::Unknown(ref s) => format!("Custom: {}", s),
                other => format!("{:?}", other),
            };
            
            if let Some(value) = item.value().text() {
                // Skip duplicates we already added
                if key_str.contains("(Raw)") && tags.iter().any(|t| t.value == value) {
                    continue;
                }
                
                tags.push(TagEntry {
                    key: key_str,
                    value: value.to_string(),
                    tag_type: tag_type.clone(),
                });
            }
        }
    }
    
    // Check other tag types too
    for tag in tagged_file.tags() {
        if Some(tag) == tagged_file.primary_tag() {
            continue; // Already processed
        }
        
        let tag_type = format!("{:?} (Secondary)", tag.tag_type());
        
        for item in tag.items() {
            if let Some(value) = item.value().text() {
                let key = format!("{:?}", item.key());
                
                tags.push(TagEntry {
                    key,
                    value: value.to_string(),
                    tag_type: tag_type.clone(),
                });
            }
        }
    }
    
    Ok(RawTags {
        file_path: file_path.to_string(),
        file_format,
        duration_seconds,
        bitrate,
        sample_rate,
        tags,
    })
}