use serde::{Deserialize, Serialize};
use rytube::Thumbnail;
use rytube::structs::{AudioQuality, VideoQuality, QualityLabel, Stream};
use reqwest::Client;


pub struct ManagedProgress(pub DownloadProgress);
pub struct ManagedVideo(pub Option<CurrentVideoInfo>);

impl ManagedVideo {
    pub fn new_video(&mut self, video: CurrentVideoInfo){
        self.0 = Some(video);
    }
}



#[derive(Serialize,Deserialize, Clone)]
pub struct DownloadableStream{
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub includes_video: bool,
    pub includes_audio: bool,
    pub extension: String,
    pub audio_quality: Option<AudioQuality>,
    pub fps: Option<u16>,
    pub quality: Option<VideoQuality>,
    pub bitrate: Option<u64>,
    pub quality_label: Option<QualityLabel>,
    pub url: String,
    pub content_length: Option<u64>
}

impl DownloadableStream {
    pub async fn from_rytube_stream(stream: Stream, client: &Client) -> Self{
        DownloadableStream {
            content_length: stream.content_length(client).await,
            width: stream.width,
            height: stream.height,
            includes_video: stream.includes_video,
            includes_audio: stream.includes_audio,
            extension: stream.mime_type.mime.subtype().to_string(),
            audio_quality: stream.audio_quality,
            fps: stream.fps,
            bitrate: stream.bitrate,
            quality: stream.quality,
            quality_label: stream.quality_label,
            url: stream.stream_url,
        }
    }
}


#[derive(Serialize,Deserialize, Clone)]
pub struct CurrentVideoInfo{
    pub id: Option<String>,
    pub title: Option<String>,
    pub views_count: Option<String>,
    pub likes_count: Option<String>,
    pub comments_count: Option<String>,
    pub previews: Vec<Thumbnail>,
    pub streams: Vec<DownloadableStream>,
    pub duration: u32
}

#[derive(Serialize,Deserialize, Clone)]
pub enum DownloadProgress{
    Idle,
    DownloadingVideo((u64, u64)),
    DownloadingAudio((u64, u64)),
    EncodingAudio((u32, u32)),
    MergingStreams((u32, u32)),
    CuttingStream((u32, u32)),
    Done,
    Canceled,
    Failed
}

impl DownloadProgress {

    pub fn add_progress(&mut self, progress: usize){
        match self {
            DownloadProgress::DownloadingVideo(tuple) | DownloadProgress::DownloadingAudio(tuple) => {
                tuple.0 += progress as u64
            },
            _ => {}
        }
    }
}

#[derive(Serialize,Deserialize, Clone)]
pub enum DownloaderError{
    InternalError(String),
    FetchingError(String),
    DownloadingError(String),
    MergingError(String),
    CuttingError(String),
    ConvertingError(String),
    FFmpegNotFound(String),
    CanceledByUser
}