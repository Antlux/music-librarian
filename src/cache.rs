use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

const CACHE_PATH: &str = "data/cache.json";

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    spotify_token: Option<String>,
    tracks: Vec<CacheTrack>
}

impl Cache {
    pub fn load_or_create() -> Self {
        File::open(CACHE_PATH)
            .and_then(|file| {
                Ok(serde_json::from_reader::<_, Self>(BufReader::new(file)).unwrap_or_default())
            }).unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize)]
pub struct CacheTrack {
    name: String,
    itunes_id: Option<String>,
    spotify_id: Option<String>,
}