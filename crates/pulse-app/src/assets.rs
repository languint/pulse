use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
struct PulseAssets;

pub struct CombinedAssets;

impl AssetSource for CombinedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        PulseAssets::get(path).map_or_else(
            || gpui_component_assets::Assets.load(path),
            |file| Ok(Some(file.data)),
        )
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut items = gpui_component_assets::Assets.list(path)?;
        items.extend(
            PulseAssets::iter().filter_map(|entry| entry.starts_with(path).then(|| entry.into())),
        );
        items.sort();
        items.dedup();
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_assets_loads_pulse_and_bundled_icons() {
        let assets = CombinedAssets;

        let pencil = assets
            .load("icons/pencil.svg")
            .expect("load pencil")
            .expect("pencil svg");
        assert!(pencil.starts_with(b"<svg"));

        let arrow = assets
            .load("icons/arrow-left.svg")
            .expect("load arrow-left")
            .expect("arrow-left svg");
        assert!(arrow.starts_with(b"<svg"));
    }

    #[test]
    fn combined_assets_errors_on_missing_icon() {
        let assets = CombinedAssets;
        let error = assets.load("icons/not-a-real-icon.svg");
        assert!(error.is_err());
    }
}
