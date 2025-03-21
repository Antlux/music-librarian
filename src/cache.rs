use std::{collections::HashSet, fs::File, io::{BufReader, BufWriter}};

use serde::{Deserialize, Serialize};

const CACHE_PATH: &str = "data/cache.json";

pub enum TrackID {
    SpotifyID(String),
    ItunesID(String)
}

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    tracks: HashSet<CacheTrack>
}

impl Cache {
    pub fn load_or_create() -> Self {
        File::open(CACHE_PATH)
            .and_then(|file| {
                Ok(serde_json::from_reader::<_, Self>(BufReader::new(file)).unwrap_or_default())
            }).unwrap_or_default()
    }

    pub fn serialize(&self) -> Result<(), std::io::Error> {
        let file = File::create(CACHE_PATH)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self).map_err(|e| e.into())
    }

    pub fn contains(&self, id: TrackID) -> bool {
        match id {
            TrackID::SpotifyID(id) => self.tracks.iter().filter_map(|t| t.spotify_id.as_ref()).find(|spotify_id| spotify_id == &&id).is_some(),
            TrackID::ItunesID(id) => self.tracks.iter().filter_map(|t| t.itunes_id.as_ref()).find(|itunes_id| itunes_id == &&id).is_some()
        }
    }

    pub fn cache_track(&mut self, name: String, spotify_id: Option<String>, itunes_id: Option<String>) {
        self.tracks.insert(CacheTrack {name, spotify_id, itunes_id});
    }

    // pub fn fetch(&self, id: TrackID) -> &CacheTrack {
    //     self.
    // }
}

#[derive(Serialize, Deserialize, Eq, Hash)]
pub struct CacheTrack {
    name: String,
    spotify_id: Option<String>,
    itunes_id: Option<String>,
}

impl PartialEq for CacheTrack {
    fn eq(&self, other: &Self) -> bool {
        let spotify = if let (Some(a), Some(b)) = (&self.spotify_id, &other.spotify_id) {a == b} else {false};
        let itunes = if let (Some(a), Some(b)) = (&self.itunes_id, &other.itunes_id) {a == b} else {false};
        spotify || itunes
    }
}