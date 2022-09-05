use ffmpeg_cli::{FfmpegBuilder, File as FFMpegFile, Parameter, Error::CanceledError};
use futures::TryStreamExt;
use std::{env, path::PathBuf};
use std::sync::{Mutex, Arc};
use crate::structs::{ManagedProgress, DownloaderError, DownloadProgress};
use std::path::Path;
use std::fs::remove_file;
use futures::future::ready;


pub fn find_ffmpeg() -> Result<PathBuf, DownloaderError>{
    let path = env::var("Path");
    match path {
        Ok(string) => {
            for dir in string.split(';') {
                let ffmpeg = Path::new(dir).join(Path::new("ffmpeg.exe"));
                if ffmpeg.is_file(){
                    return Ok(ffmpeg);
                }
            }
        },
        Err(_) => {}
    };
    let current_directory = env::current_dir();
    if current_directory.is_ok(){
        let ffmpeg = current_directory.unwrap().join(Path::new("ffmpeg.exe"));
        if ffmpeg.is_file(){
            return Ok(ffmpeg);
        }
    }
    Err(DownloaderError::FFmpegNotFound(format!("Can't find ffmpeg neither in PATH, nor in current directory")))
}

pub async fn merge_videos(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
    progress: Arc<Mutex<ManagedProgress>>
) -> Result<String, DownloaderError>{
    let ffmpeg_path = find_ffmpeg()?;
    let builder = 
        FfmpegBuilder::new(ffmpeg_path.to_str().unwrap())
            .input(FFMpegFile::new(video_path))
            .input(FFMpegFile::new(audio_path))
            .output(
                FFMpegFile::new(output_path)
                .option(Parameter::KeyValue("c:v", "copy"))
                .option(Parameter::KeyValue("c:a", "aac"))
            );
    let ffmpeg = builder.run().await.unwrap();

    let merging_result = ffmpeg
        .progress
        .try_for_each(|x| {
            match progress.lock(){
                Err(_) => (),
                Ok(progress) => match progress.0 {
                    DownloadProgress::Canceled => {
                        return ready(Err(CanceledError));
                    },
                    _ => ()
                }
            }
            let duration_done = x.out_time;
            if duration_done.is_some(){
                progress.lock().unwrap().0.set_ffmpeg_progress(duration_done.unwrap().as_secs() as u32)
            }
            ready(Ok(()))
        })
        .await;
    
    match merging_result {
        Ok(_) => (),
        Err(CanceledError) => return Err(DownloaderError::CanceledByUser),
        Err(err) => return Err(DownloaderError::MergingError(err.to_string()))
    }

    let output = ffmpeg.process.wait_with_output().unwrap();

    if Path::new(video_path).is_file(){
        let _ = remove_file(Path::new(video_path));
    }
    if Path::new(audio_path).is_file(){
        let _ = remove_file(Path::new(audio_path));
    }

    match output.status.code(){
        Some(code) if code == 0 => Ok(output_path.to_string()),
        Some(code) => Err(DownloaderError::MergingError(format!("FFMpeg exited with status code {}", code))),
        None => Err(DownloaderError::MergingError(format!("FFMpeg exited with no status code"))),
    }
}

pub async fn cut_video(
    video_path: &str,
    start_seconds: u32,
    end_seconds: u32,
    output_path: &str,
    progress: Arc<Mutex<ManagedProgress>>
) -> Result<String, DownloaderError>{
    let start_seconds = start_seconds.to_string();
    let end_seconds = end_seconds.to_string();
    let ffmpeg_path = find_ffmpeg()?;
    let builder = 
        FfmpegBuilder::new(ffmpeg_path.to_str().unwrap())
            .input(FFMpegFile::new(video_path))
            .output(
                FFMpegFile::new(output_path)
                .option(Parameter::KeyValue("ss", &start_seconds))
                .option(Parameter::KeyValue("to", &end_seconds))
            );
    let ffmpeg = builder.run().await.unwrap();

    let cutting_result = ffmpeg
        .progress
        .try_for_each(|x| {
            match progress.lock(){
                Err(_) => (),
                Ok(progress) => match progress.0 {
                    DownloadProgress::Canceled => {
                        return ready(Err(CanceledError));
                    },
                    _ => ()
                }
            }
            let duration_done = x.out_time;
            if duration_done.is_some(){
                progress.lock().unwrap().0.set_ffmpeg_progress(duration_done.unwrap().as_secs() as u32)
            }
            ready(Ok(()))
        })
        .await;

    match cutting_result {
        Ok(_) => (),
        Err(CanceledError) => return Err(DownloaderError::CanceledByUser),
        Err(err) => return Err(DownloaderError::MergingError(err.to_string()))
    }

    let output = ffmpeg.process.wait_with_output().unwrap();

    if Path::new(video_path).is_file(){
        let _ = remove_file(Path::new(video_path));
    }

    match output.status.code(){
        Some(code) if code == 0 => Ok(output_path.to_string()),
        Some(code) => Err(DownloaderError::CuttingError(format!("FFMpeg exited with status code {}", code))),
        None => Err(DownloaderError::CuttingError(format!("FFMpeg exited with no status code"))),
    }
}

pub async fn convert_to_mp3(
    video_path: &str,
    progress: Arc<Mutex<ManagedProgress>>
) -> Result<String, DownloaderError>{
    let ffmpeg_path = find_ffmpeg()?;
    let output_path = video_path[..video_path.rfind('.').unwrap()].to_string()+".mp3";
    let builder = 
        FfmpegBuilder::new(ffmpeg_path.to_str().unwrap())
            .input(FFMpegFile::new(video_path))
            .output(
                FFMpegFile::new(&output_path)
                .option(Parameter::Single("vn"))
            );
    let ffmpeg = builder.run().await.unwrap();

    let converting_result = ffmpeg
        .progress
        .try_for_each(|x| {
            match progress.lock(){
                Err(_) => (),
                Ok(progress) => match progress.0 {
                    DownloadProgress::Canceled => {
                        return ready(Err(CanceledError));
                    },
                    _ => ()
                }
            }

            let duration_done = x.out_time;
            if duration_done.is_some(){
                progress.lock().unwrap().0.set_ffmpeg_progress(duration_done.unwrap().as_secs() as u32)
            }
            ready(Ok(()))
        })
        .await;
    
    match converting_result {
        Ok(_) => (),
        Err(CanceledError) => return Err(DownloaderError::CanceledByUser),
        Err(err) => return Err(DownloaderError::MergingError(err.to_string()))
    }

    let output = ffmpeg.process.wait_with_output().unwrap();
        
    if Path::new(video_path).is_file(){
        let _ = remove_file(Path::new(video_path));
    }
        
    match output.status.code(){
        Some(code) if code == 0 => Ok(output_path),
        Some(code) => Err(DownloaderError::ConvertingError(format!("FFMpeg exited with status code {}", code))),
        None => Err(DownloaderError::ConvertingError(format!("FFMpeg exited with no status code"))),
    }
}