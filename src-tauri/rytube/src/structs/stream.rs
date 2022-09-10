use crate::{Error, unwrap_or_return_error};
use crate::decryption::singature_decryption::Cipher;
use serde::{Deserialize, Serialize};
use url::Url;
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::BufWriter, io::AsyncWriteExt};
use super::stream_elements::{MimeType, VideoQuality, AudioQuality, QualityLabel, SignatureCipher};
use super::raw_stream::RawStream;


#[derive(Debug, Deserialize, Serialize)]
pub struct Stream{
    pub itag: u64,
    #[serde(with = "crate::deserializers::mime_type")]
    pub mime_type: MimeType,
    pub bitrate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    content_length: Option<u64>,
    pub quality: Option<VideoQuality>,
    pub fps: Option<u16>,
    pub quality_label: Option<QualityLabel>,
    pub average_bitrate: Option<u64>,
    pub audio_quality: Option<AudioQuality>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u8>,
    pub includes_audio: bool,
    pub includes_video: bool,
    pub stream_url: String,
}

impl Stream {
    pub(crate) fn from_raw_stream(raw: RawStream, cipher: &Option<Cipher>) -> Result<Self, Error>{
        Ok(
            Stream {
                itag: raw.itag,
                bitrate: raw.bitrate,
                width: raw.width,
                height: raw.height,
                content_length: raw.content_length,
                quality: raw.quality,
                fps: raw.fps,
                quality_label: raw.quality_label,
                average_bitrate: raw.average_bitrate,
                audio_sample_rate: raw.audio_sample_rate,
                audio_channels: raw.audio_channels,
                includes_audio: if raw.audio_channels.is_none() && 
                                   raw.audio_sample_rate.is_none() && 
                                   raw.audio_quality.is_none()
                                { false }else{ true },
                includes_video: if raw.width.is_none() &&
                                   raw.height.is_none() &&
                                   raw.mime_type.mime.type_().as_str() != "video"
                                { false }else{ true },
                audio_quality: raw.audio_quality,
                mime_type: raw.mime_type,
                stream_url: unwrap_or_return_error!(
                                signature_cipher_to_url(raw.signature_cipher, cipher),
                                Error::SignatureDecryptionError(format!("Signature can't be decrypted"))
                            )
            }
        )
    }

    pub async fn content_length(&self, client: &Client) -> Option<u64>{
        if self.content_length.is_some(){
            return self.content_length;
        }
        let head_response = client
                            .head(&self.stream_url)
                            .send()
                            .await
                            .ok()?
                            .error_for_status()
                            .ok()?;
        let header = head_response.headers().get(reqwest::header::CONTENT_LENGTH)?;
        header.to_str().ok()?.parse().ok()
    }

    pub async fn download_to(&self, path: &Path, client: &Client) -> Result<String, Error>{
        let path = PathBuf::from(path)
                    .with_extension(self.mime_type.mime.subtype().to_string());
        let file = unwrap_or_return_error!(
            File::create(&path).await,
            Error::InternalError(format!("Can't create file at {}", path.display()))
        );
        let mut buffer = BufWriter::with_capacity(8*1024*1024, file);

        let content_length = self.content_length(client).await;
        if content_length.is_none(){
            return Err(Error::HTTPError(format!("Can't get stream content-length")));
        }
        let content_length = content_length.unwrap();
        let mut downloaded: u64 = 0;
        let chunk_size: u64 = 1024*1024;

        while downloaded < content_length{
            let end_range = std::cmp::min(downloaded+chunk_size, content_length);
            let response = unwrap_or_return_error!(
                client
                .get(self.stream_url.clone())
                .header(reqwest::header::RANGE, format!("bytes={}-{}", downloaded, end_range)) 
                .send()
                .await,
                Error::HTTPError(format!("Error while sending request to stream url"))
            );

            let response = response.error_for_status();
            if response.is_err(){
                return Err(
                    Error::HTTPError(
                        format!(
                            "Stream url returned {}",
                            response.unwrap_err().status().unwrap()
                        )
                    )
                );
            }
            let response = response.unwrap();
            let chunk = unwrap_or_return_error!(
                response.bytes().await,
                Error::HTTPError(format!("Error while reading stream bytes"))
            );
            if buffer.write_all(&chunk).await.is_err(){
                return Err(Error::InternalError(format!("Error while writing stream to file")));
            }
            downloaded+=chunk.len() as u64;
        };
        if buffer.flush().await.is_err(){
            return Err(Error::InternalError(format!("Error while saving file at {}", path.display())));
        };
        Ok(path.to_string_lossy().to_string())
    }
}

fn signature_cipher_to_url(
    signature_cipher: SignatureCipher,
    cipher: &Option<Cipher>,
) -> Result<String, Error>{
    match signature_cipher {
        SignatureCipher::Url(url) => {
            if url_contains_signature(&url) {
                Ok(url)
            }else{
                Err(Error::InternalError(format!("No url or signature cipher in stream")))
            }
        },
        SignatureCipher::Signature{url, s}=> {
            match cipher {
                None => Err(Error::SignatureDecryptionError(format!("Can't decrypt signature without cipher"))),
                Some(cipher) => {
                    let mut parsed_url = unwrap_or_return_error!(
                        Url::parse(&url),
                        Error::InternalError(format!("Stream url is invalid"))
                    );
                    match cipher.decrypt_signature(s){
                        Err(err) => Err(err),
                        Ok(s) => {
                            parsed_url.query_pairs_mut().append_pair("sig", &s);
                            //N-query decryption is not implemented yet, so downloading will be slow if url is not signed already
                            Ok(parsed_url.to_string())
                        },
                    }
                }
            }
        }
    }
}

fn url_contains_signature(url: &str) -> bool {
    url.contains("signature") || (url.contains("&sig=") || url.contains("&lsig="))
}
