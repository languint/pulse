use std::io::Cursor;

use image::{GenericImageView, ImageFormat};
use pulse_model::ThumbnailSize;

use crate::error::LibraryError;

pub fn generate_thumbnail(data: &[u8], size: u32) -> Result<Vec<u8>, LibraryError> {
    let image =
        image::load_from_memory(data).map_err(|source| LibraryError::ArtworkDecode { source })?;

    let thumbnail = image.resize_to_fill(size, size, image::imageops::FilterType::Triangle);

    let mut output = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut output, ImageFormat::Jpeg)
        .map_err(|source| LibraryError::ArtworkEncode { source })?;

    Ok(output.into_inner())
}

pub fn image_dimensions(data: &[u8]) -> Result<(u32, u32), LibraryError> {
    let image =
        image::load_from_memory(data).map_err(|source| LibraryError::ArtworkDecode { source })?;

    Ok(image.dimensions())
}

pub fn source_extension(data: &[u8]) -> &'static str {
    match image::guess_format(data) {
        Ok(ImageFormat::Png) => "png",
        Ok(ImageFormat::Jpeg) => "jpg",
        Ok(ImageFormat::Gif) => "gif",
        Ok(ImageFormat::WebP) => "webp",
        Ok(ImageFormat::Bmp) => "bmp",
        _ => "bin",
    }
}

pub fn generate_all_thumbnails(
    data: &[u8],
    cache: &super::ArtworkCache,
    content_hash: &str,
) -> Result<Vec<(ThumbnailSize, std::path::PathBuf)>, LibraryError> {
    let mut thumbnails = Vec::with_capacity(ThumbnailSize::all().len());

    for size in ThumbnailSize::all() {
        let path = cache.thumbnail_path(content_hash, size);
        if !path.exists() {
            let bytes = generate_thumbnail(data, size.pixels())?;
            super::ArtworkCache::write_if_missing(&path, &bytes).map_err(|source| {
                LibraryError::Io {
                    path: path.clone(),
                    source,
                }
            })?;
        }

        thumbnails.push((size, path));
    }

    Ok(thumbnails)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_png() -> Result<Vec<u8>, image::ImageError> {
        let mut buffer = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            32,
            32,
            image::Rgb([120, 80, 200]),
        ))
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
        Ok(buffer)
    }

    #[test]
    fn generates_square_jpeg_thumbnail() -> Result<(), Box<dyn std::error::Error>> {
        let png = sample_png()?;
        let thumb = generate_thumbnail(&png, 64)?;
        let decoded = image::load_from_memory(&thumb)?;

        if decoded.width() != 64 || decoded.height() != 64 {
            return Err("expected 64x64 thumbnail".into());
        }
        Ok(())
    }
}
