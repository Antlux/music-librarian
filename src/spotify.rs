use rspotify::{scopes, AuthCodeSpotify, Credentials, OAuth};

pub fn init() -> AuthCodeSpotify {
    let creds = Credentials::from_env().expect("Could not get credentials");
    let oauth = OAuth::from_env(scopes!("playlist-modify-private", "playlist-modify-public")).expect("Could not get oauth");
    AuthCodeSpotify::new(creds, oauth)
}