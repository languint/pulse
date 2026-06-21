use lofty::picture::PictureType;
use lofty::tag::Tag;

#[must_use]
pub fn extract_cover_art(tag: &Tag) -> Option<Vec<u8>> {
    tag.get_picture_type(PictureType::CoverFront)
        .or_else(|| tag.pictures().first())
        .map(|picture| picture.data().to_vec())
        .filter(|data| !data.is_empty())
}
