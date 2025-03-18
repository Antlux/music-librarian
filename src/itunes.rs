use std::{collections::{HashMap, HashSet}, fmt::Display, fs::File, io::BufReader, path::Path};

use elementtree::Element;

#[derive(Debug)]
pub enum Error {
    XMLFileError(std::io::Error),
    XMLParsingError(elementtree::Error),
    XMLLibraryError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            &Error::XMLFileError(e) => write!(f, "Encountered error reading XML file: {}", e),
            &Error::XMLParsingError(e) => write!(f, "Encountered error parsing XML text: {}", e),
            &Error::XMLLibraryError(e) => write!(f, "Encountered error constructing XML library: {}", e)
        }
    }
}

impl std::error::Error for Error {
    
}


pub struct Library {
    tracks: HashSet<Track>,
    playlists: Vec<Playlist>,
}

impl Library {
    pub fn from_xml<P: AsRef<Path>>(path: P) -> Result<Self, Error>{
        let file = File::open(path).map_err(|e| Error::XMLFileError(e))?;
        let buf_rdr = BufReader::new(file);
        let library_root = elementtree::Element::from_reader(buf_rdr).map_err(|e| Error::XMLParsingError(e))?;

        let library = library_root.find("dict")
            .ok_or(Error::XMLLibraryError("Could not find library dict".into()))?;

        let library_data = gather_element_data(&library)?;
        
        let tracks = library_data
            .get("Tracks")
            .ok_or(Error::XMLLibraryError("Could not find tracks dict".into()))?
            .find_all("dict")
            .filter_map(|e| gather_element_data(e).ok())
            .filter_map(|data| Track::from_element_data(data).ok())
            .collect::<HashSet<_>>();

        let playlists = library_data
            .get("Playlists")
            .ok_or(Error::XMLLibraryError("Could not find playlists array".into()))?
            .find_all("dict")
            .filter_map(|e| gather_element_data(e).ok())
            .filter_map(|data| Playlist::from_element_data(data).ok())
            .collect::<Vec<_>>();


        Ok(Library{tracks, playlists})
    }


    pub fn tracks(&self) -> &HashSet<Track> {&self.tracks}
    pub fn playlists(&self) -> &Vec<Playlist> {&self.playlists}
}

#[derive(Eq, Hash, Debug)]
pub struct Track {
    name: String,
    artist: String,
    id: usize,
    persistent_id: String,
}

impl Track {
    fn from_element_data(data: HashMap<String, &Element>) -> Result<Self, Error> {
        let name = data.get("Name".into()).and_then(|o| Some(o.text())).expect("Cannot find track name.").into();
        let artist = data.get("Artist".into()).and_then(|o| Some(o.text())).expect("Cannot find track artist.").into();
        let id = data.get("Track ID".into()).and_then(|o| Some(o.text())).and_then(|o| o.parse::<usize>().ok()).expect("Cannot find track id");
        let persistent_id = data.get("Persistent ID".into()).and_then(|o| Some(o.text())).expect("Cannot find track artist.").into();
        Ok(Self { name, artist, id, persistent_id})
    }

    pub fn name(&self) -> String {self.name.clone()}
    pub fn artist(&self) -> String {self.artist.clone()}
    pub fn id(&self) -> usize {self.id}
    pub fn persistent_id(&self) -> String {self.persistent_id.clone()}
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.persistent_id == other.persistent_id
    }
}


fn gather_element_data<'a>(element:  &Element) -> Result<HashMap<String, &Element>, Error> {
    let data = element
        .children()
        .collect::<Vec<_>>()
        .chunks_exact(2)
        .filter_map(|c| {
            let k = c.get(0)?.text().to_owned();
            let v = c.get(1)?.to_owned();
            Some((k, v))
        })
        .fold(HashMap::new(), |mut h, (k, v)| {
            h.insert(k, v);
            h
        });

    (!data.is_empty()).then_some(data).ok_or(Error::XMLLibraryError("Failed to gather node data".to_string()))
}

#[derive(Debug)]
pub struct Playlist {
    name: String,
    description: String,
    track_ids: Vec<usize>
}

impl Playlist {
    fn from_element_data<'a>(data: HashMap<String, &Element>) -> Result<Self, Error> {
        let name = data.get("Name".into()).and_then(|o| Some(o.text())).ok_or(Error::XMLLibraryError("Could not find playlist name".into()))?.to_owned();
        let description = data.get("Description".into()).and_then(|o| Some(o.text())).ok_or(Error::XMLLibraryError("Could not find playlist description".into()))?.to_owned();
        let track_ids = data.get("Playlist Items".into())
            .and_then(|e| Some(e.children()))
            .ok_or(Error::XMLLibraryError("Could not find playlist tracks".into()))?
            .filter_map(|e| e.children().find_map(|e| e.text().parse::<usize>().ok()))
            .collect::<Vec<_>>();
        Ok(Self { name, description, track_ids})
    }

    pub fn name(&self) -> String {self.name.clone()}
    pub fn description(&self) -> String {self.description.clone()}
    pub fn track_ids(&self) -> &Vec<usize> {&self.track_ids}
}