use std::{sync::Arc, thread, time::Duration};

use console::Term;
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use music_librarian::{cache::Cache, itunes, spotify};
use rspotify::{model::{FullTrack, Id, SearchResult, SearchType}, prelude::BaseClient};


pub enum Actions {

}


#[tokio::main]
async fn main() {
    let spotify = Arc::new(spotify::init().await);
    
    let mut cache = Cache::load_or_create();

    let lib = itunes::Library::from_xml("data/Library.xml").expect("Could not get itunes Library");

    let cache_absent_tracks = lib.tracks().into_iter().filter_map(|track| {
        (!cache.contains(music_librarian::cache::TrackID::ItunesID(track.persistent_id()))).then_some(track)
    }).collect::<Vec<_>>();

    let bar = ProgressBar::new(cache_absent_tracks.len() as u64)
        .with_style(ProgressStyle::with_template("[{pos:>3}/{len:3}] {wide_msg} {bar:100}").unwrap());

    let mut handles = vec![];

    for track in cache_absent_tracks {
        bar.inc(1);
        bar.set_message(format!("Fetching track: {} by {}", track.name(), track.artist()));
        let search_prompt = format!("{} {}", track.name(), track.artist());
        let spotify = spotify.clone(); 
        let h = tokio::spawn(async move {spotify.search(&search_prompt, SearchType::Track, None, None, Some(5), None).await});
        handles.push((track, h));

        thread::sleep(Duration::from_millis(650));
    }

    bar.finish_with_message("Fetched all tracks.");

    let term = Term::stdout();

    let bar = ProgressBar::new(handles.len() as u64)
        .with_style(ProgressStyle::with_template("[{pos:>3}/{len:3}] {wide_msg} {bar:100}").unwrap());

    for (itunes_track, handle) in handles {
        bar.inc(1);
        bar.set_message(format!("Associating track: {} by {}", itunes_track.name(), itunes_track.artist()));
        thread::sleep(Duration::from_millis(10));
        let response = handle.await.unwrap();
        if let Ok(search_results) = response {
            if let SearchResult::Tracks(page) = search_results {
                if does_result_match(itunes_track, page.items.first().unwrap()) {
                    let name = itunes_track.name();
                    let spotify_id = Some(page.items.first().unwrap().id.as_ref().unwrap().uri());
                    let itunes_id = Some(itunes_track.persistent_id());
                    cache.cache_track(name, spotify_id, itunes_id);
                } else if let Some(spotify_track) = prompt_result_match(&page.items) {
                    let name = itunes_track.name();
                    let spotify_id = Some(spotify_track.id.as_ref().unwrap().uri());
                    let itunes_id = Some(itunes_track.persistent_id());
                    cache.cache_track(name, spotify_id, itunes_id);
                    term.clear_last_lines(2).unwrap();
                }
            }
        }

        
    }

    bar.finish_with_message("Associated all tracks.");

    cache.serialize().unwrap();

}


fn does_result_match(itunes_track: &itunes::Track, spotify_track: &FullTrack) -> bool {
    let a = itunes_track.artist().to_lowercase().replace(" ", "").trim().to_owned();
    let b = spotify_track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join("").to_lowercase().replace(" ", "").trim().to_owned();
    b.contains(&a) || a.contains(&b)
}

fn prompt_result_match<'a>(spotify_tracks: &'a Vec<FullTrack>) -> Option<&'a FullTrack> {
    let track_prompts = spotify_tracks.iter().map(|t| format!("{} by {}", t.name, t.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", "))).collect::<Vec<_>>(); 
    let selection = Select::new()
        .with_prompt("Pick (or not) closest result")
        .items(&track_prompts)
        .interact_opt()
        .unwrap();
    selection.and_then(|idx| spotify_tracks.get(idx))
}