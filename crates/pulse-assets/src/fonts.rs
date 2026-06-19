pub const INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");

pub fn get(path: &str) -> Option<&'static [u8]> {
    match path {
        "fonts/Inter.ttf" => Some(INTER),
        _ => None,
    }
}
