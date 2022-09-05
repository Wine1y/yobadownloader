#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]


mod structs;
mod commands;
mod downloader;
mod ffmpeg_utils;
use std::sync::{Mutex, Arc};

use commands::{
    try_get_video_id, load_new_video, get_current_video, 
    get_default_path, start_downloading, get_downloading_progress,
    ffmpeg_installed, cancel_downloading};
use structs::{ManagedVideo, ManagedProgress, DownloadProgress};


fn main() {
  tauri::Builder::default()
    .manage(Mutex::new(ManagedVideo(None)))
    .manage(Arc::new(Mutex::new(ManagedProgress(DownloadProgress::Idle))))
    .invoke_handler(tauri::generate_handler![
        try_get_video_id,
        load_new_video,
        get_current_video,
        start_downloading,
        get_default_path,
        get_downloading_progress,
        ffmpeg_installed,
        cancel_downloading
    ])
    .run(tauri::generate_context!())
    .expect("running tauri application");
}