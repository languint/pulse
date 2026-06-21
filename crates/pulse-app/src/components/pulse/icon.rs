use std::sync::Arc;

use gpui::{App, Hsla, Image, ImageFormat, IntoElement, Pixels, Styled, img};
use gpui_component::ActiveTheme;

const PULSE_PATH: &str = "M6 31.5H15.75L22.25 41L32 22L41.75 41L48.25 31.5H58";

pub fn pulse_logo(size: Pixels, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let svg = pulse_logo_svg(theme.primary, theme.primary_hover, theme.secondary);
    let image = Arc::new(Image::from_bytes(ImageFormat::Svg, svg.into_bytes()));

    img(image).size(size).flex_shrink_0()
}

fn pulse_logo_svg(primary: Hsla, primary_shade: Hsla, surface: Hsla) -> String {
    let surface = hex_color(surface);
    let gradient_start = hex_color(primary);
    let gradient_end = hex_color(primary_shade);

    format!(
        r#"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
<g clip-path="url(#pulse_logo_clip)">
<rect width="64" height="64" rx="4" fill="{surface}"/>
<path d="{PULSE_PATH}" stroke="url(#pulse_logo_gradient)" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>
</g>
<defs>
<linearGradient id="pulse_logo_gradient" x1="32" y1="22" x2="32" y2="41" gradientUnits="userSpaceOnUse">
<stop stop-color="{gradient_start}"/>
<stop offset="1" stop-color="{gradient_end}"/>
</linearGradient>
<clipPath id="pulse_logo_clip">
<rect width="64" height="64" fill="white"/>
</clipPath>
</defs>
</svg>"#
    )
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions
)]
fn hex_color(color: Hsla) -> String {
    let rgb = color.to_rgb();
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb.r * 255.0).round() as u8,
        (rgb.g * 255.0).round() as u8,
        (rgb.b * 255.0).round() as u8
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_component::Theme;

    #[test]
    fn pulse_logo_svg_uses_theme_colors() {
        let theme = Theme::default();
        let svg = pulse_logo_svg(theme.primary, theme.primary_hover, theme.secondary);

        assert!(svg.contains("linearGradient"));
        assert!(svg.contains(PULSE_PATH));
        assert!(svg.contains(&hex_color(theme.secondary)));
        assert!(svg.contains(&hex_color(theme.primary)));
        assert!(svg.contains(&hex_color(theme.primary_hover)));
    }
}
