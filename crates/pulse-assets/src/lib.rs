use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

pub mod fonts;
pub mod icons;

mod assets_macro;

pub struct PulseAssetSource;

impl AssetSource for PulseAssetSource {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let asset = fonts::get(path).or_else(|| icons::get(path));

        Ok(asset.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        // It isn't actually required to load the assets, so we can return an empty Vec.
        Ok(vec![])
    }
}
