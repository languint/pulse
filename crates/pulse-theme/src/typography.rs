use gpui::{FontFallbacks, FontFeatures, FontWeight, Pixels, SharedString, px};

#[derive(Debug, Clone, Copy)]
pub struct TypographyTokens {
    pub font_family: &'static str,

    pub display: TextStyle,
    pub title: TextStyle,
    pub body: TextStyle,
    pub label: TextStyle,
    pub caption: TextStyle,
}

impl TypographyTokens {
    pub const DEFAULT: Self = Self {
        font_family: "Inter",

        display: TextStyle {
            size: px(32.0),
            weight: FontWeight::BOLD,
        },
        title: TextStyle {
            size: px(20.0),
            weight: FontWeight::SEMIBOLD,
        },
        body: TextStyle {
            size: px(14.0),
            weight: FontWeight::NORMAL,
        },
        label: TextStyle {
            size: px(12.0),
            weight: FontWeight::MEDIUM,
        },
        caption: TextStyle {
            size: px(11.0),
            weight: FontWeight::NORMAL,
        },
    };
}

impl TypographyTokens {
    pub fn font(&self, style: TextStyle) -> gpui::Font {
        gpui::Font {
            family: SharedString::new(self.font_family),
            features: FontFeatures::default(),
            fallbacks: Some(FontFallbacks::default()),
            weight: style.weight,
            style: gpui::FontStyle::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub size: Pixels,
    pub weight: FontWeight,
}
