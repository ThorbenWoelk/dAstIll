use super::{media_asset_key, media_asset_kind_slug};
use crate::models::MediaAssetKind;

#[test]
fn source_audio_asset_key_is_stable() {
    assert_eq!(
        media_asset_kind_slug(MediaAssetKind::SourceAudio),
        "source-audio"
    );
    assert_eq!(
        media_asset_key("podcast:episode:episode-1", MediaAssetKind::SourceAudio),
        "media-assets/podcast:episode:episode-1/source-audio.json"
    );
}
