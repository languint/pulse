use gpui::{Hsla, rgba};

#[derive(Debug, Clone, Copy)]
pub struct TextColors {
    pub primary: Hsla,
    pub secondary: Hsla,
    pub muted: Hsla,
    pub disabled: Hsla,
}

impl Default for TextColors {
    fn default() -> Self {
        Self {
            primary: Hsla::from(rgba(0xccccccff)),
            secondary: Hsla::from(rgba(0x9d9d9dff)),
            muted: Hsla::from(rgba(0x6e7681ff)),
            disabled: Hsla::from(rgba(0xcccccc80)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorTokens {
    pub background: Hsla,
    pub surface: Hsla,
    pub surface_variant: Hsla,

    pub text: TextColors,

    pub primary: Hsla,
    pub secondary: Hsla,

    pub success: Hsla,
    pub warning: Hsla,
    pub error: Hsla,

    pub border: Hsla,
    pub border_variant: Hsla,
}

impl ColorTokens {
    pub fn default_dark() -> Self {
        Self {
            background: Hsla::from(rgba(0x1f1f1fff)),
            surface: Hsla::from(rgba(0x181818ff)),
            surface_variant: Hsla::from(rgba(0x202020ff)),

            primary: Hsla::from(rgba(0x0078d4ff)),
            secondary: Hsla::from(rgba(0x179fffff)),

            success: Hsla::from(rgba(0x2ea043ff)),
            warning: Hsla::from(rgba(0xcca700ff)),
            error: Hsla::from(rgba(0xf14c4cff)),

            border: Hsla::from(rgba(0x2b2b2bff)),
            border_variant: Hsla::from(rgba(0x454545ff)),

            text: TextColors::default(),
        }
    }
}
