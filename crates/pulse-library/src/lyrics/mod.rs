mod cache;
mod fetch;
mod load;
mod parse;

pub use cache::LyricsCache;
pub use fetch::{LyricsFetchError, LyricsLookup, fetch_lrclib_lyrics};
pub use load::{load_local_lyrics, load_sidecar_lyrics, sidecar_candidates};
pub use parse::parse_lrc;
