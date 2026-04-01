use std::path::PathBuf;
use std::process::Command;

use super::extract_ffmpeg::extract_ffmpeg;

pub fn mux_audio_video(video: &PathBuf, audio: &PathBuf, output: &PathBuf) -> std::io::Result<()> {
    let ffmpeg_path = extract_ffmpeg()?; // write embedded ffmpeg to temp
    let status = Command::new(ffmpeg_path)
        .args(&[
            "-y",
            "-i",
            video.to_str().unwrap(),
            "-i",
            audio.to_str().unwrap(),
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            output.to_str().unwrap(),
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "FFmpeg failed",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mux_audio_video() {
        let video_path = dirs::video_dir()
            .unwrap_or(PathBuf::from("."))
            .join("test2_noaudio.mp4");
        let audio_path = PathBuf::from(
            "C:\\Users\\claire\\rust_scripts\\download_song\\analyse_aubio\\push_up.wav",
        );
        let output_path = dirs::video_dir()
            .unwrap_or(PathBuf::from("."))
            .join("test2_audio.mp4");

        let result = mux_audio_video(&video_path, &audio_path, &output_path);
        assert!(result.is_ok());

        // delete the test video file after the test
        std::fs::remove_file(output_path).unwrap();
    }
}
