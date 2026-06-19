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

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let assets = match path {
            "fonts" => vec!["fonts/Inter.ttf"],
            "icons" => vec!["icons/x.svg"],

            _ => vec![],
        };

        Ok(assets.into_iter().map(SharedString::from).collect())
    }
}
