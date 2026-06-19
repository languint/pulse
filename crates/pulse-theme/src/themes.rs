use crate::{
    AnimationTokens, ColorTokens, PulseTheme, RadiusTokens, SpacingTokens, TypographyTokens,
};

#[must_use]
pub fn pulse_dark() -> PulseTheme {
    PulseTheme {
        name: "Pulse Dark",
        colors: ColorTokens::default_dark(),
        radii: RadiusTokens::DEFAULT,
        spacing: SpacingTokens::DEFAULT,
        typography: TypographyTokens::DEFAULT,
        animation: AnimationTokens::DEFAULT,
    }
}
