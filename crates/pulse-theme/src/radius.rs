#[derive(Debug, Clone, Copy)]
pub struct RadiusTokens {
    pub unit: f32,

    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl RadiusTokens {
    pub const DEFAULT: Self = {
        let unit = 3.0;

        Self {
            unit,

            xs: unit,
            sm: unit * 2.0,
            md: unit * 3.0,
            lg: unit * 4.0,
            xl: unit * 6.0,
        }
    };

    pub const DEFAULT_COMPACT: Self = {
        let unit = 2.0;

        Self {
            unit,

            xs: unit,
            sm: unit * 2.0,
            md: unit * 3.0,
            lg: unit * 4.0,
            xl: unit * 6.0,
        }
    };
}
