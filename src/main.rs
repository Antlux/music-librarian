fn main() {
    let itunes_library = music_librarian::itunes::Library::from_xml("data/Library.xml").unwrap();
    for track in itunes_library.tracks() {
        println!("{}", track.name())
    }
    for playlist in itunes_library.playlists() {
        println!("{}", playlist.name())
    }
}
