use std::{
    io::Write,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
};

use super::extract_ffmpeg::extract_ffmpeg;

/// Incremental video exporter using an embedded FFmpeg binary.
/// Frames must be RGB24 bytes.
pub struct FFmpegCmdExporter {
    child: Child,
    stdin: Option<ChildStdin>,
    width: u32,
    height: u32,
    frame_index: usize,
}

impl FFmpegCmdExporter {
    /// Create a new exporter.
    pub fn new(width: u32, height: u32, fps: u32, output: &PathBuf) -> std::io::Result<Self> {
        let ffmpeg_path = extract_ffmpeg()?;
        let mut child = Command::new(ffmpeg_path)
            .args(&[
                "-y",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-video_size",
                &format!("{}x{}", width, height),
                "-framerate",
                &fps.to_string(),
                "-i",
                "-", // input from stdin
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                output.to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();

        Ok(Self {
            child,
            stdin: Some(stdin),
            width,
            height,
            frame_index: 0,
        })
    }

    /// Push a single RGB24 frame.
    /// `rgb` must be exactly width*height*3 bytes.
    pub fn push_frame(&mut self, rgb: &[u8]) -> std::io::Result<()> {
        if rgb.len() != (self.width * self.height * 3) as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Incorrect frame size",
            ));
        }

        self.stdin.as_mut().unwrap().write_all(rgb)?;
        self.frame_index += 1;
        Ok(())
    }

    /// Finish writing the video and wait for FFmpeg to exit.
    pub fn finish(&mut self) -> std::io::Result<()> {
        self.stdin.take();
        let status = self.child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "FFmpeg failed",
            ))
        }
    }
}
