use ffmpeg_cli::{FfmpegBuilder, File as FFMpegFile, Parameter};
use std::os::windows::process::CommandExt;
use std::{env, path::PathBuf};
use crate::structs::DownloaderError;
use std::path::Path;
use std::fs::remove_file;

const CREATE_NO_WINDOW: u32 = 0x08000000;

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
    output_path: &str
) -> Result<String, DownloaderError>{
    let ffmpeg_path = find_ffmpeg()?;
    let builder = 
        FfmpegBuilder::new(ffmpeg_path.to_str().unwrap())
            .option(Parameter::Single("y"))
            .input(FFMpegFile::new(video_path))
            .input(FFMpegFile::new(audio_path))
            .output(
                FFMpegFile::new(output_path)
                .option(Parameter::KeyValue("c:v", "copy"))
                .option(Parameter::KeyValue("c:a", "aac"))
            );
    let ffmpeg = builder
                    .to_command()
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
    if ffmpeg.is_err(){
        return Err(DownloaderError::MergingError("Failed to spawn ffmpeg process".to_owned()))
    }
    let output = ffmpeg.unwrap().wait_with_output().unwrap();

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
    output_path: &str
) -> Result<String, DownloaderError>{
    let start_seconds = start_seconds.to_string();
    let end_seconds = end_seconds.to_string();
    let ffmpeg_path = find_ffmpeg()?;
    let builder = 
        FfmpegBuilder::new(ffmpeg_path.to_str().unwrap())
            .option(Parameter::Single("y"))
            .input(FFMpegFile::new(video_path))
            .output(
                FFMpegFile::new(output_path)
                .option(Parameter::KeyValue("ss", &start_seconds))
                .option(Parameter::KeyValue("to", &end_seconds))
            );
    let ffmpeg = builder
                    .to_command()
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
    if ffmpeg.is_err(){
        return Err(DownloaderError::CuttingError("Failed to spawn ffmpeg process".to_owned()))
    }
    let output = ffmpeg.unwrap().wait_with_output().unwrap();

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
) -> Result<String, DownloaderError>{
    let ffmpeg_path = find_ffmpeg()?;
    let output_path = video_path[..video_path.rfind('.').unwrap()].to_string()+".mp3";
    let builder = 
        FfmpegBuilder::new(ffmpeg_path.to_str().unwrap())
            .option(Parameter::Single("y"))
            .input(FFMpegFile::new(video_path))
            .output(
                FFMpegFile::new(&output_path)
                .option(Parameter::Single("vn"))
            );
    let ffmpeg = builder
                    .to_command()
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
    if ffmpeg.is_err(){
        return Err(DownloaderError::ConvertingError("Failed to spawn ffmpeg process".to_owned()))
    }
    let output = ffmpeg.unwrap().wait_with_output().unwrap();
        
    if Path::new(video_path).is_file(){
        let _ = remove_file(Path::new(video_path));
    }
        
    match output.status.code(){
        Some(code) if code == 0 => Ok(output_path),
        Some(code) => Err(DownloaderError::ConvertingError(format!("FFMpeg exited with status code {}", code))),
        None => Err(DownloaderError::ConvertingError(format!("FFMpeg exited with no status code"))),
    }
}