use super::{MimeType, VideoQuality, QualityLabel, AudioQuality, SignatureCipher};
use serde::{Deserialize};
use serde_with::json::JsonString;
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawStream{
    pub itag: u64,
    #[serde(with = "crate::deserializers::mime_type")]
    pub mime_type: MimeType,
    pub bitrate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde_as(as = "Option<JsonString>")]
    pub content_length: Option<u64>,
    pub quality: Option<VideoQuality>,
    pub fps: Option<u16>,
    pub quality_label: Option<QualityLabel>,
    pub average_bitrate: Option<u64>,
    pub audio_quality: Option<AudioQuality>,
    #[serde_as(as = "Option<JsonString>")]
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u8>,
    #[serde(flatten, with = "crate::deserializers::signature_cipher")]
    pub signature_cipher: SignatureCipher
}