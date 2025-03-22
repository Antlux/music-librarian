use std::{collections::HashSet, fmt::Display, sync::{Arc, Mutex}, thread, time::Duration};

use console::Term;
use dialoguer::Select;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use music_librarian::{cache::{self, Cache}, itunes::{self, Library}, spotify};
use rspotify::{model::{FullTrack, Id, SearchResult, SearchType}, prelude::OAuthClient};


pub enum Error {

}


pub enum Actions {
    TransferLibrary,
    TransferPlaylist,
}

impl Display for Actions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::TransferLibrary => write!(f, "Transfer music library"),
            &Self::TransferPlaylist => write!(f, "Transfer music playlist")
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Services {
    Itunes,
    Spotify,
}

impl Display for Services {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::Itunes => write!(f, "Itunes"),
            &Self::Spotify => write!(f, "Spotify")
        }
    }
}


#[tokio::main]
async fn main() {
    let term = Term::stdout();
    term.write_line("<<<    Music Librarian    >>>").unwrap();
    let actions = [Actions::TransferLibrary, Actions::TransferPlaylist];
    let selection = Select::new().items(&actions).with_prompt("Choose action to perform").interact().unwrap();
    match actions[selection] {
        Actions::TransferLibrary => prompt_transfer_library(&term).await,
        Actions::TransferPlaylist => {}
    }
}


async fn prompt_transfer_library(term: &Term) {
    let from_services = [Services::Itunes, Services::Spotify];
    let from_selection = Select::new().items(&from_services).with_prompt("From").interact().unwrap();
    let from = from_services[from_selection];
    term.clear_last_lines(1).unwrap();
    let to_services = from_services.into_iter().filter(|a| a != &from).collect::<Vec<_>>();
    let to_selection = Select::new().items(&to_services).with_prompt("From ".to_string() + &from.to_string() + " ===> To").interact().unwrap();
    let to = to_services[to_selection];

    match (from, to) {
        (Services::Itunes, Services::Spotify) => {
            term.write_line("Initializing spotify client...").unwrap();
            let spotify = spotify::init().await;
            term.write_line("Successfully initiated spotify client!").unwrap();
            let file_handle = rfd::AsyncFileDialog::new()
                .add_filter("Itunes library", &["xml"])
                .pick_file().await.expect("No file was picked");
            let itunes_library = Library::from_xml(file_handle.path()).expect("Could not parse library");
            transfer_itunes_spotify(term, spotify, itunes_library).await
        },
        _ => {}
    }
}

async fn transfer_itunes_spotify(term: &Term, spotify: impl OAuthClient + 'static, itunes_library: Library) {
    let cache = Cache::load_or_create();
    let cache_absent_tracks = itunes_library.tracks().iter().cloned().filter(|t| !cache.contains(cache::TrackID::ItunesID(t.persistent_id()))).collect::<HashSet<_>>();
    let len = cache_absent_tracks.len() as u64;
    let len_len = len.to_string().len().to_string();

    let bar = ProgressBar::new(len)
        .with_style(ProgressStyle::with_template(&("[{pos:>".to_string() + &len_len + "}/{len:" + &len_len + "}] {bar:30} {msg}")).unwrap());

    let spotify = Arc::new(spotify);
    let cache = Arc::new(Mutex::new(cache));

    let mut handles = vec![];

    for itunes_track in cache_absent_tracks.into_iter() {
        bar.set_message(format!("Fetching track missing from cache: '{}' by {}", itunes_track.name(), itunes_track.artist()));
        let spotify = spotify.clone();
        let cache = cache.clone();
        let search_prompt = format!("{} {}", itunes_track.name(), itunes_track.artist());
        let handle = tokio::spawn(async move {
            let response = spotify.search(&search_prompt, SearchType::Track, None, None, Some(25), None).await;
            match response {
                Ok(SearchResult::Tracks(page)) => {
                    if let Some(spotify_track) = page.items.first() {
                        if does_result_match(&itunes_track, spotify_track) {
                            let mut c = cache.lock().unwrap();
                            let name = itunes_track.name();
                            let spotify_id = Some(spotify_track.id.as_ref().unwrap().uri());
                            let itunes_id = Some(itunes_track.persistent_id());
                            c.cache_track(name, spotify_id, itunes_id);
                            c.serialize().expect("Could not serialize cache");
                            
                            return None;
                        }
                    }
                    Some((itunes_track, page))
                },
                _ => None
            }
        });
        handles.push(handle);
        bar.inc(1);
        thread::sleep(Duration::from_millis(650));
    }

    bar.finish_with_message("Fetched all tracks missing from cache.");

    let responses = join_all(handles).await;
    let non_associated_tracks = responses
        .into_iter()
        .filter_map(|r| r.ok().and_then(|s| s.and_then(|a| Some(a))))
        .collect::<Vec<_>>();
    
    let len = non_associated_tracks.len() as u64;
    let len_len = len.to_string().len().to_string();
    let bar = ProgressBar::new(len)
        .with_style(ProgressStyle::with_template(&("[{pos:>".to_string() + &len_len + "}/{len:" + &len_len + "}] {bar:30} {msg}")).unwrap());

    for (itunes_track, search_result) in non_associated_tracks {
        bar.set_message(format!("Associating '{}' by {}", itunes_track.name(), itunes_track.artist()));
        if let Some(spotify_track) = prompt_result_match(&search_result.items) {
            let mut c = cache.lock().unwrap();
            let name = itunes_track.name();
            let spotify_id = Some(spotify_track.id.as_ref().unwrap().uri());
            let itunes_id = Some(itunes_track.persistent_id());
            c.cache_track(name, spotify_id, itunes_id);
            c.serialize().expect("Could not serialize cache");
            term.clear_last_lines(2).unwrap();
        } else {
            term.clear_last_lines(1).unwrap();
        }
        bar.inc(1);
    }
    
    bar.finish_with_message("Associated all tracks");

}


fn does_result_match(itunes_track: &itunes::Track, spotify_track: &FullTrack) -> bool {
    let a = itunes_track.artist().to_lowercase().replace(" ", "").trim().to_owned();
    let b = spotify_track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join("").to_lowercase().replace(" ", "").trim().to_owned();
    b.contains(&a) || a.contains(&b)
}

fn prompt_result_match<'a>(spotify_tracks: &'a Vec<FullTrack>) -> Option<&'a FullTrack> {
    let track_prompts = spotify_tracks.iter()
        .map(|t| format!("'{}' by {}", {
            let n = &t.name;
            if n.len() > 20 {
                if let Some(slice) = n.get(0..17) {
                    slice.to_string() + "..."
                } else {
                    n.clone()
                }
            } else {
                n.clone()
            }
        }, 
        {
            let artists = t.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ");
            if let Some(url) = t.external_urls.get("spotify") {
                vec![artists, url.clone()].join(" - ")
            } else {
                artists
            }
        })).collect::<Vec<_>>(); 
    let selection = Select::new()
        .with_prompt("Pick (or not) closest result")
        .items(&track_prompts)
        .max_length(5)
        .interact_opt()
        .unwrap();
    selection.and_then(|idx| spotify_tracks.get(idx))
}