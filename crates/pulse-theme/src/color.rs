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
            primary: Hsla::from(rgba(0xcccc_ccff)),
            secondary: Hsla::from(rgba(0x9d9d_9dff)),
            muted: Hsla::from(rgba(0x6e76_81ff)),
            disabled: Hsla::from(rgba(0xcccc_cc80)),
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
    #[must_use]
    pub fn default_dark() -> Self {
        Self {
            background: Hsla::from(rgba(0x1f1f_1fff)),
            surface: Hsla::from(rgba(0x1818_18ff)),
            surface_variant: Hsla::from(rgba(0x2020_20ff)),

            primary: Hsla::from(rgba(0x0078_d4ff)),
            secondary: Hsla::from(rgba(0x179f_ffff)),

            success: Hsla::from(rgba(0x2ea0_43ff)),
            warning: Hsla::from(rgba(0xcca7_00ff)),
            error: Hsla::from(rgba(0xf14c_4cff)),

            border: Hsla::from(rgba(0x2b2b_2bff)),
            border_variant: Hsla::from(rgba(0x4545_45ff)),

            text: TextColors::default(),
        }
    }
}
