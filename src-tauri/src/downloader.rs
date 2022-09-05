use crate::structs::{DownloadableStream, ManagedProgress, DownloadProgress, DownloaderError};
use crate::ffmpeg_utils;

use std::path::Path;
use std::sync::{Mutex, Arc};
use reqwest::{Client, header, cookie};
use tokio::{fs::File, io::BufWriter, io::AsyncWriteExt};
use std::fs::remove_file;


fn get_headers() -> header::HeaderMap{
    let mut headers = header::HeaderMap::new();

    headers.insert(header::ACCEPT_LANGUAGE, "en-US,en".parse().unwrap());
    headers.insert(header::USER_AGENT, "Mozilla/5.0".parse().unwrap());
    headers.insert(header::CONNECTION, "keep-alive".parse().unwrap());

    headers
}

fn get_cookies() -> cookie::Jar{
    let cookie = "CONSENT=YES+; Path=/; Domain=youtube.com; Secure; Expires=Fri, 01 Jan 2038 00:00:00 GMT;";
    let url = "https://youtube.com".parse().unwrap();

    let jar = cookie::Jar::default();
    jar.add_cookie_str(cookie, &url);
    jar
}


pub async fn download_stream(
    path: &str,
    stream: &DownloadableStream,
    progress: Arc<Mutex<ManagedProgress>>
) -> Result<String, DownloaderError>{
    let real_path = path[..path.rfind('.').unwrap()].to_string()+"."+&stream.extension;
    let file = File::create(&real_path).await;
    if file.is_err(){
        return Err(DownloaderError::InternalError(format!("Can't create/open file at {}", &real_path))); 
    };
    let file = file.unwrap();

    let client = Client::builder()
    .default_headers(get_headers())
    .cookie_provider(std::sync::Arc::new(get_cookies()))
    .build();
    if client.is_err(){
        return Err(DownloaderError::DownloadingError(format!("HTTP Client cannot be initialized")));
    }
    let client = client.unwrap();

    let mut buffer = BufWriter::with_capacity(8*1024*1024, file);
    let content_length = stream.content_length;
    if content_length.is_none(){
        return Err(DownloaderError::InternalError(format!("Can't get stream content-length")));
    }
    let content_length = content_length.unwrap();
    let mut downloaded: u64 = 0;
    let chunk_size: u64 = 1024*1024;

    while downloaded < content_length{
        match progress.lock() {
            Err(_) => (),
            Ok(progress) => match progress.0 {
                DownloadProgress::Canceled => {
                    return Err(DownloaderError::CanceledByUser); 
                },
                _ => ()
            }
        }
        let end_range = std::cmp::min(downloaded+chunk_size, content_length);
        let response = client
                       .get(stream.url.clone())
                       .header(header::RANGE, format!("bytes={}-{}", downloaded, end_range)) 
                       .send()
                       .await;
        if response.is_err(){
            return Err(DownloaderError::DownloadingError(format!("Error while sending request to stream url")));
        }
        let response = response.unwrap().error_for_status();

        if response.is_err(){
            return Err(
                DownloaderError::DownloadingError(
                    format!(
                        "Stream url returned {}",
                        response.unwrap_err().status().unwrap()
                    )
                )
            );
        }
        let response = response.unwrap();
        let chunk = response.bytes().await;
        if chunk.is_err(){
            return Err(DownloaderError::DownloadingError(format!("Error while reading stream bytes")))
        }
        let chunk = chunk.unwrap();
        if buffer.write_all(&chunk).await.is_err(){
            return Err(DownloaderError::DownloadingError(format!("Error while writing stream to file at {}", &real_path)));
        }
        downloaded+=chunk.len() as u64;
        progress.lock().unwrap().0.add_progress(chunk.len())
    }
    if buffer.flush().await.is_err(){
        return Err(DownloaderError::DownloadingError(format!("Error while saving file at {}", &real_path)));
    };
    Ok(real_path)
}

pub async fn download_one_stream(
    stream: &DownloadableStream,
    save_path: &str,
    progress: Arc<Mutex<ManagedProgress>>
)   -> Result<String, DownloaderError>{
    progress.lock().unwrap().0 = 
        DownloadProgress::DownloadingVideo((0, stream.content_length.unwrap_or(0)));
    download_stream(save_path, stream, progress).await
}

pub async fn download_and_merge_audio(
    stream: &DownloadableStream,
    save_path: &str,
    audio_stream: &DownloadableStream,
    video_duration: u32,
    progress: Arc<Mutex<ManagedProgress>>
) -> Result<String, DownloaderError>{
    let video_path = save_path[..save_path.rfind('\\').unwrap()].to_string()+"\\temp_video."+&stream.extension;
    let audio_path = save_path[..save_path.rfind('\\').unwrap()].to_string()+"\\temp_audio."+&audio_stream.extension;
    let output_path = save_path[..save_path.rfind('.').unwrap()].to_string()+"."+&stream.extension;
    
    progress.lock().unwrap().0 = 
        DownloadProgress::DownloadingVideo((0, stream.content_length.unwrap_or(0)));
    let video_saved_to = download_stream(&video_path, stream, progress.clone()).await?;

    progress.lock().unwrap().0 = 
        DownloadProgress::DownloadingAudio((0, audio_stream.content_length.unwrap_or(0)));
    let audio_saved_to = download_stream(&audio_path, audio_stream, progress.clone()).await?;

    progress.lock().unwrap().0 = DownloadProgress::MergingStreams((0, video_duration));
    ffmpeg_utils::merge_videos(
        &video_saved_to,
        &audio_saved_to,
        &output_path,
        progress.clone()).await
}

pub(crate) fn clear_downloaded_files(paths: Vec<String>){
    paths
    .into_iter()
    .for_each(|path| {
        let path = Path::new(&path);
        if path.is_file(){
            let _ =remove_file(path);
        }
    })
}



