use rspotify::{prelude::{BaseClient, OAuthClient}, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};

const TOKEN_CACHE: &str = "data/spotify_token.txt";

pub async fn init() -> impl OAuthClient {
    let creds = Credentials::from_env().expect("Could not get credentials");
    let oauth = OAuth::from_env(scopes!("playlist-modify-private", "playlist-modify-public")).expect("Could not get oauth");

    if let Ok(token) = Token::from_cache(TOKEN_CACHE) {
        AuthCodeSpotify::from_token_with_config(token, creds, oauth, Config::default())
    } else {
        let spotify = AuthCodeSpotify::new(creds, oauth);
        let url = spotify.get_authorize_url(true).expect("Could not get url");
        spotify.prompt_for_token(&url).await.expect("Could not get token");
        if let Some(t) = spotify.get_token().lock().await.unwrap().as_ref() {
            t.write_cache(TOKEN_CACHE).expect("Could not write cache");
        }
        spotify
    }
}