**YobaDownloader**

Simple YouTube downloader made with Rust and Tauri, requires FFmpeg to be installed for some features.
The code is messy, but i don't have time to refactor it right now.

Known issues:
1. Extremely slow downloading of some videos.
2. Visible ffmpeg window.
3. Broken icon.
4. Inability to go to the previous screen if the download immediately ends with an error.
5. FFmpeg throws an error if the output file already exists.
