mod animation;
mod color;
mod radius;
mod spacing;
mod typography;

pub mod themes;

pub use animation::AnimationTokens;
pub use color::ColorTokens;
pub use radius::RadiusTokens;
pub use spacing::SpacingTokens;
pub use typography::TypographyTokens;

#[derive(Debug, Clone, Copy)]
pub struct PulseTheme {
    pub name: &'static str,

    pub colors: ColorTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub animation: AnimationTokens,
}
