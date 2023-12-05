use std::sync::{Mutex, Arc};
use std::env::current_dir;
use regex::Regex;
use lazy_static::lazy_static;
use tauri::State;
use directories::UserDirs;
use futures::{future::join_all};


use crate::{downloader, ffmpeg_utils};
use crate::structs::{DownloadProgress, ManagedProgress, DownloaderError};
use rytube;
use crate::structs::{CurrentVideoInfo, DownloadableStream, ManagedVideo};


#[tauri::command]
pub fn try_get_video_id(link: &str) -> Option<String>{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(youtube\.com/watch\?v=([a-zA-Z_0-9-]{11}))|(youtu\.be/([a-zA-Z_0-9-]{11}))")
                                .unwrap();
    }
    match RE.captures(link) {
        None => None,
        Some(captures) => {
            match captures.get(2) {
                Some(id_match) => Some(id_match.as_str().to_string()),
                None => match captures.get(4) {
                    Some(id_match) => Some(id_match.as_str().to_string()),
                    None => None
                }
            }
        } 
    }
}


#[tauri::command]
pub async fn load_new_video<'r>(
    managed_video: State<'r, Mutex<ManagedVideo>>,
    video_id: &str
) -> Result<bool, DownloaderError>{
    let video = match rytube::Video::from_video_id(video_id.to_owned()).await {
        Err(_) => {return Err(DownloaderError::FetchingError(format!("Can't get video with id {}", video_id)))},
        Ok(video) => video
    };

    let downloadable_streams = 
        video.streams
        .into_iter()
        .map(|stream| async {
            DownloadableStream::from_rytube_stream(stream, &video.client).await
        });
    let downloadable_streams = join_all(downloadable_streams).await;

    let current_video = CurrentVideoInfo{
        id: video.id,
        title: video.title,
        views_count: video.views,
        likes_count: video.likes,
        comments_count: video.comments,
        previews: video.thumbnails,
        streams: downloadable_streams,
        duration: video.length_seconds.unwrap_or(0)
    };

    managed_video.lock().unwrap().new_video(current_video);
    Ok(true)
}

#[tauri::command]
pub async fn get_current_video<'r>(
    managed_video: State<'r, Mutex<ManagedVideo>>
) -> Result<CurrentVideoInfo, DownloaderError>{
    match managed_video.lock(){
        Err(_) => Err(DownloaderError::InternalError(format!("Can't get current video right now"))),
        Ok(current_video) => {
            match current_video.0.as_ref() {
                None => Err(DownloaderError::InternalError(format!("No video loaded"))),
                Some(video) => Ok(video.clone())
            }
        }
    }
}

#[tauri::command]
pub async fn start_downloading<'r>(
    managed_progress: State<'r, Arc<Mutex<ManagedProgress>>>,
    stream: DownloadableStream,
    time_line: [u32; 3],
    save_path: String,
    audio_stream: Option<DownloadableStream>
) -> Result<String, DownloaderError>{

    if (time_line[0] > 0 || time_line[1] < time_line[2]) || save_path.ends_with(".mp3") || audio_stream.is_some(){
        let ffmpeg_path = ffmpeg_utils::find_ffmpeg();
        if ffmpeg_path.is_err(){
            managed_progress.lock().unwrap().0 = DownloadProgress::Failed;
            return Err(ffmpeg_path.unwrap_err()); 
        }
    }

    let mut probably_created = vec![
        save_path.clone(),
        save_path[..save_path.rfind('.').unwrap()].to_string()+"."+&stream.extension,
        save_path[..save_path.rfind('\\').unwrap()].to_string()+"\\video-uncutted."+&stream.extension,
        save_path[..save_path.rfind('\\').unwrap()].to_string()+"\\temp_video."+&stream.extension,
    ];

    if audio_stream.is_some(){
        probably_created.push(save_path[..save_path.rfind('\\').unwrap()].to_string()+"\\temp_audio."+&audio_stream.as_ref().unwrap().extension)
    }

    let temp_path: String;
    if time_line[0] > 0 || time_line[1] < time_line[2]{
        temp_path = save_path[..save_path.rfind('\\').unwrap()].to_string()+"\\video-uncutted."+&stream.extension;
    }else{
        temp_path = save_path.clone();
    }

    let result = match audio_stream {
        None => {
            downloader::download_one_stream(&stream, &temp_path, Arc::clone(&managed_progress)).await
        },
        Some(audio_stream) =>{
            downloader::download_and_merge_audio(&stream, &temp_path, &audio_stream, time_line[2], Arc::clone(&managed_progress)).await
        }
    };

    let result = match result{
        Err(DownloaderError::CanceledByUser) => {
            downloader::clear_downloaded_files(probably_created);
            return Err(DownloaderError::CanceledByUser);
        }
        Err(err) => Err(err),
        Ok(video_path) => {
            if save_path.ends_with(".mp3") && !video_path.ends_with(".mp3"){
                managed_progress.lock().unwrap().0 = DownloadProgress::EncodingAudio((0, time_line[2]));
                let res = ffmpeg_utils::convert_to_mp3(&video_path).await;
                if res.is_ok(){
                    managed_progress.lock().unwrap().0 = DownloadProgress::EncodingAudio((time_line[2], time_line[2]));
                }
                res
            }else{
                Ok(video_path)
            }
        }
    };

    match result {
        Err(DownloaderError::CanceledByUser) => {
            downloader::clear_downloaded_files(probably_created);
            return Err(DownloaderError::CanceledByUser);
        }
        Err(_) => {
            downloader::clear_downloaded_files(probably_created);
            managed_progress.lock().unwrap().0 = DownloadProgress::Failed;
            result
        }
        Ok(video_path) => {
            if time_line[0] > 0 || time_line[1] < time_line[2]{
                managed_progress.lock().unwrap().0 = DownloadProgress::CuttingStream((0, time_line[1]-time_line[0]));
                let cutting_res = ffmpeg_utils::cut_video(
                    &video_path,
                    time_line[0],
                    time_line[1],
                    &save_path).await;
                match cutting_res{
                    Err(DownloaderError::CanceledByUser) => {
                        downloader::clear_downloaded_files(probably_created);
                        return Err(DownloaderError::CanceledByUser);
                    },
                    Err(err) => {
                        downloader::clear_downloaded_files(probably_created);
                        managed_progress.lock().unwrap().0 = DownloadProgress::Failed;
                        Err(err)
                    },
                    Ok(path) => {
                        managed_progress.lock().unwrap().0 = DownloadProgress::CuttingStream((time_line[1]-time_line[0], time_line[1]-time_line[0]));
                        managed_progress.lock().unwrap().0 = DownloadProgress::Done;
                        Ok(path)
                    }
                }
            }else{
                managed_progress.lock().unwrap().0 = DownloadProgress::Done;
                Ok(video_path)
            }
        },
        
    }
}

#[tauri::command]
pub async fn get_downloading_progress<'r>(
    managed_progress: State<'r, Arc<Mutex<ManagedProgress>>>
) -> Result<DownloadProgress, DownloaderError>{
    match managed_progress.lock() {
        Ok(download_progress) => Ok(download_progress.0.clone()),
        Err(_) => Err(DownloaderError::InternalError(format!("Can't get downloading progress right now")))
    }
}

#[tauri::command]
pub fn get_default_path() -> Option<String>{
    if let Some(user_dirs) = UserDirs::new() {
        let desktop_path = user_dirs.desktop_dir();
        if desktop_path.is_some(){
            return Some(desktop_path.unwrap().to_string_lossy().to_string());
        }
    }
    return Some(current_dir().ok()?.to_string_lossy().to_string());
}

#[tauri::command]
pub fn ffmpeg_installed() -> bool{
    match ffmpeg_utils::find_ffmpeg(){
        Ok(_) => true,
        Err(_) => false
    }
}

#[tauri::command]
pub fn cancel_downloading<'r>(
    managed_progress: State<'r, Arc<Mutex<ManagedProgress>>>
) -> Result<bool, DownloaderError>{
    match managed_progress.lock() {
        Ok(mut download_progress) => {
            download_progress.0 = DownloadProgress::Canceled;
            Ok(true)
        },
        Err(_) => Err(DownloaderError::InternalError(format!("Can't cancel downloading")))
    }
}