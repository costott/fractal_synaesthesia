use std::path::PathBuf;

use ffmpeg_next::format::Pixel;
use ffmpeg_next::software::scaling::{context::Context as Scaler, flag::Flags as ScaleFlags};
use ffmpeg_next::{self as ffmpeg, packet::Mut};

/// Handles exporting a sequence of RGB frames to a video file using FFmpeg.
pub struct VideoExporterFFmpeg {
    fmt_ctx: ffmpeg::format::context::Output,
    stream_index: usize,
    encoder: ffmpeg::codec::encoder::Video,
    scaler: Scaler,

    width: u32,
    height: u32,
    frame_index: i64,
    fps: i32,
}
impl VideoExporterFFmpeg {
    pub fn new(path: &PathBuf, width: u32, height: u32, fps: u32) -> Result<Self, ffmpeg::Error> {
        ffmpeg::init()?;

        let mut fmt_ctx = ffmpeg::format::output(path)?;

        let codec =
            ffmpeg::encoder::find(ffmpeg::codec::Id::H264).ok_or(ffmpeg::Error::EncoderNotFound)?;

        // Check container format flags before we borrow fmt_ctx mutably
        let global_header = fmt_ctx
            .format()
            .flags()
            .contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER);

        // Create stream (metadata only)
        let mut stream = fmt_ctx.add_stream(codec)?;
        let stream_index = stream.index();

        // Create encoder context explicity
        let encoder_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
        let mut video = encoder_ctx.encoder().video()?;

        video.set_width(width);
        video.set_height(height);
        video.set_format(ffmpeg::format::Pixel::YUV420P);
        video.set_time_base((1, fps as i32));

        let mut x264_opts = ffmpeg::Dictionary::new();
        x264_opts.set("preset", "medium"); // required by libx264

        if global_header {
            video.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
        }

        // 4. Open encoder
        let opened_encoder = video.open_with(x264_opts)?;

        // 5. Copy encoder params to stream
        stream.set_parameters(&opened_encoder);

        // create scaler for RGB24 to encoder pixel format (YUV420P)
        let scaler = Scaler::get(
            Pixel::RGB24,
            width,
            height,
            ffmpeg::format::Pixel::YUV420P,
            width,
            height,
            ScaleFlags::BILINEAR,
        )?;

        fmt_ctx.write_header()?;

        Ok(Self {
            fmt_ctx,
            stream_index,
            encoder: opened_encoder,
            scaler,
            width,
            height,
            frame_index: 0,
            fps: fps as i32,
        })
    }

    pub fn push_frame(&mut self, rgb: &[u8]) -> Result<(), ffmpeg::Error> {
        // create source RGB frame and copy bytes
        let mut src = ffmpeg::util::frame::Video::new(Pixel::RGB24, self.width, self.height);

        // expect rgb.len() == width * height * 3
        let expected = (self.width * self.height * 3) as usize;
        if rgb.len() != expected {
            return Err(ffmpeg::Error::InvalidData);
        }

        // prepare line sizes and data buffer, then copy RGB into the first plane
        let linesize = src.stride(0) as usize;
        let data0 = src.data_mut(0);
        if data0.len() >= rgb.len() {
            data0[..rgb.len()].copy_from_slice(rgb);
        } else {
            // fallback to row-by-row copy using linesize
            let row_bytes = (self.width * 3) as usize;
            for y in 0..(self.height as usize) {
                let src_off = y * row_bytes;
                let dst_off = y * linesize;
                data0[dst_off..dst_off + row_bytes]
                    .copy_from_slice(&rgb[src_off..src_off + row_bytes]);
            }
        }

        // destination YUV frame
        let mut dst = ffmpeg::util::frame::Video::new(
            ffmpeg::format::Pixel::YUV420P,
            self.width,
            self.height,
        );

        // convert RGB -> YUV
        self.scaler.run(&src, &mut dst)?;

        dst.set_pts(Some(self.frame_index));
        self.encoder.send_frame(&dst)?;

        loop {
            let mut packet = ffmpeg::codec::packet::Packet::empty();
            if self.encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(self.stream_index);
                packet.rescale_ts(
                    (1, self.fps),
                    self.fmt_ctx.stream(self.stream_index).unwrap().time_base(),
                );
                let ret = unsafe {
                    ffmpeg::sys::av_interleaved_write_frame(
                        self.fmt_ctx.as_mut_ptr(),
                        packet.as_mut_ptr(),
                    )
                };
                if ret < 0 {
                    return Err(ffmpeg::Error::from(ret));
                }
            } else {
                break;
            }
        }

        self.frame_index += 1;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), ffmpeg::Error> {
        self.encoder.send_eof()?;

        loop {
            let mut packet = ffmpeg::codec::packet::Packet::empty();
            if self.encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(self.stream_index);
                packet.rescale_ts(
                    (1, self.fps),
                    self.fmt_ctx.stream(self.stream_index).unwrap().time_base(),
                );
                let ret = unsafe {
                    ffmpeg::sys::av_interleaved_write_frame(
                        self.fmt_ctx.as_mut_ptr(),
                        packet.as_mut_ptr(),
                    )
                };
                if ret < 0 {
                    return Err(ffmpeg::Error::from(ret));
                }
            } else {
                break;
            }
        }

        self.fmt_ctx.write_trailer()?;
        Ok(())
    }
}

// quick test suite
#[cfg(test)]
mod tests {
    use super::VideoExporterFFmpeg;
    use std::fs;

    #[test]
    fn test_video_exporter_ffmpeg() {
        let output_path = std::path::PathBuf::from("test_output.mp4");
        let width = 320;
        let height = 240;
        let fps = 30;
        let frame_count = fps * 2; // 2 seconds

        let mut exporter = VideoExporterFFmpeg::new(&output_path, width, height, fps)
            .expect("Failed to create exporter");

        for i in 0..frame_count {
            // Create a simple test pattern (gradient)
            let mut frame_data = vec![0u8; (width * height * 3) as usize];
            for y in 0..height {
                for x in 0..width {
                    let offset = ((y * width + x) * 3) as usize;
                    frame_data[offset] = (x % 256) as u8; // R
                    frame_data[offset + 1] = (y % 256) as u8; // G
                    frame_data[offset + 2] = ((i * 10) % 256) as u8; // B
                }
            }
            exporter
                .push_frame(&frame_data)
                .expect("Failed to push frame");
        }

        exporter.finish().expect("Failed to finish export");

        // Check if file was created
        assert!(output_path.exists());

        // Clean up
        fs::remove_file(output_path).expect("Failed to delete test output file");
    }
}
