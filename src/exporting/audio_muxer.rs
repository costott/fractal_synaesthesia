use std::path::PathBuf;

use ffmpeg_next as ffmpeg;
use ffmpeg_next::packet::Mut;

pub fn mux_audio_to_video(
    video_path: &PathBuf,
    audio_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), ffmpeg::Error> {
    // Open inputs
    let mut video_in = ffmpeg::format::input(video_path)?;
    let mut audio_in = ffmpeg::format::input(audio_path)?;

    // Create output
    let mut out = ffmpeg::format::output(output_path)?;

    // Find best streams (capture indices and params, then drop borrows)
    let video_stream = video_in
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let video_stream_index = video_stream.index();
    let video_time_base = video_stream.time_base();

    let audio_stream = audio_in
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let audio_stream_index = audio_stream.index();
    let audio_parameters = audio_stream.parameters().clone();

    drop(video_stream);
    drop(audio_stream);

    // Copy video stream (no re-encoding)
    let mut out_video = out.add_stream(
        video_in
            .stream(video_stream_index)
            .unwrap()
            .parameters()
            .id(),
    )?;
    out_video.set_parameters(video_in.stream(video_stream_index).unwrap().parameters());
    let out_video_index = out_video.index();
    drop(out_video);

    // Add audio encoder to output
    let codec_id = ffmpeg::codec::Id::AAC;
    let codec = ffmpeg::encoder::find(codec_id).ok_or(ffmpeg::Error::EncoderNotFound)?;
    let audio_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);

    let mut audio_enc = audio_ctx.encoder().audio()?;
    audio_enc.set_rate(44100);
    audio_enc.set_channel_layout(ffmpeg::ChannelLayout::STEREO);
    audio_enc.set_format(ffmpeg::format::Sample::F32(
        ffmpeg::format::sample::Type::Planar,
    ));
    audio_enc.set_time_base((1, 44100));

    let mut audio_enc = audio_enc.open_as(codec)?;
    let mut out_audio = out.add_stream(codec)?;
    out_audio.set_parameters(&audio_enc);
    let out_audio_index = out_audio.index();
    drop(out_audio);

    // Write output header
    out.write_header()?;

    // Copy video packets
    for (stream, mut packet) in video_in.packets() {
        if stream.index() == video_stream_index {
            packet.set_stream(out_video_index);
            packet.rescale_ts(stream.time_base(), video_time_base);
            let ret = unsafe {
                ffmpeg::sys::av_interleaved_write_frame(out.as_mut_ptr(), packet.as_mut_ptr())
            };
            if ret < 0 {
                return Err(ffmpeg::Error::from(ret));
            }
        }
    }

    // Decode + encode audio
    let decoder_ctx = ffmpeg::codec::context::Context::from_parameters(audio_parameters)?;
    let mut decoder = decoder_ctx.decoder().audio()?;
    let mut resampler = ffmpeg::software::resampling::Context::get(
        decoder.format(),
        decoder.channel_layout(),
        decoder.rate(),
        audio_enc.format(),
        audio_enc.channel_layout(),
        audio_enc.rate(),
    )?;
    let mut decoded_frame = ffmpeg::util::frame::Audio::empty();

    for (_, pkt) in audio_in.packets() {
        decoder.send_packet(&pkt)?;
        while decoder.receive_frame(&mut decoded_frame).is_ok() {
            let mut resampled_frame = ffmpeg::util::frame::Audio::empty();
            resampler.run(&decoded_frame, &mut resampled_frame)?;
            audio_enc.send_frame(&resampled_frame)?;

            // Drain packets
            loop {
                let mut enc_pkt = ffmpeg::codec::packet::Packet::empty();
                if audio_enc.receive_packet(&mut enc_pkt).is_ok() {
                    enc_pkt.set_stream(out_audio_index);
                    enc_pkt.rescale_ts(
                        audio_enc.time_base(),
                        out.stream(out_audio_index).unwrap().time_base(),
                    );
                    let ret = unsafe {
                        ffmpeg::sys::av_interleaved_write_frame(
                            out.as_mut_ptr(),
                            enc_pkt.as_mut_ptr(),
                        )
                    };
                    if ret < 0 {
                        return Err(ffmpeg::Error::from(ret));
                    }
                } else {
                    break;
                }
            }
        }
    }

    // Flush decoder
    decoder.send_eof()?;
    audio_enc.send_eof()?;
    loop {
        let mut pkt = ffmpeg::codec::packet::Packet::empty();
        if audio_enc.receive_packet(&mut pkt).is_ok() {
            pkt.set_stream(out_audio_index);
            pkt.rescale_ts(
                audio_enc.time_base(),
                out.stream(out_audio_index).unwrap().time_base(),
            );
            let ret = unsafe {
                ffmpeg::sys::av_interleaved_write_frame(out.as_mut_ptr(), pkt.as_mut_ptr())
            };
            if ret < 0 {
                return Err(ffmpeg::Error::from(ret));
            }
        } else {
            break;
        }
    }

    // Write trailer
    out.write_trailer()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mux_audio_to_video() {
        let video_path = dirs::video_dir()
            .unwrap_or(PathBuf::from("."))
            .join("test2_noaudio.mp4");
        let audio_path =
            PathBuf::from("C:\\Users\\claire\\rust_scripts\\download_song\\push_up.wav");
        let output_path = dirs::video_dir()
            .unwrap_or(PathBuf::from("."))
            .join("test2_audio.mp4");

        let result = mux_audio_to_video(&video_path, &audio_path, &output_path);
        assert!(result.is_ok());
    }
}
