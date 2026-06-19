use gpui::Rems;

#[derive(Debug, Clone, Copy)]
pub struct SpacingTokens {
    pub unit: Rems,

    pub xs: Rems,
    pub sm: Rems,
    pub md: Rems,
    pub lg: Rems,
    pub xl: Rems,
}

impl SpacingTokens {
    pub const DEFAULT: Self = {
        let unit = Rems(0.25);

        Self {
            unit,

            xs: Rems(unit.0),
            sm: Rems(unit.0 * 2.0),
            md: Rems(unit.0 * 3.0),
            lg: Rems(unit.0 * 4.0),
            xl: Rems(unit.0 * 6.0),
        }
    };
}
