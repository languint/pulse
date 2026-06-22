mod artwork;
mod config;
mod error;
mod library;
mod lyrics;
mod roots;
mod scan;
mod store;
mod watch;

pub use artwork::ArtworkCache;
pub use config::LibraryConfig;
pub use error::LibraryError;
pub use library::MusicLibrary;
pub use lyrics::{
    LyricsCache, LyricsFetchError, LyricsLookup, fetch_lrclib_lyrics, load_local_lyrics,
    load_sidecar_lyrics, parse_lrc, sidecar_candidates,
};
pub use roots::resolve_roots;
pub use scan::ScanSummary;
pub use store::LibraryStore;
