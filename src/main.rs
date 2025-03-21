use music_librarian::{itunes, spotify};
use rspotify::prelude::OAuthClient;

#[tokio::main]
async fn main() {
    let spotify = spotify::init();
    let url = spotify.get_authorize_url(true).expect("Could not get url");
    spotify.prompt_for_token(&url).await.expect("Could not get token");

    let lib = itunes::Library::from_xml("data/Library.xml").expect("Could not get itunes Library");
}
