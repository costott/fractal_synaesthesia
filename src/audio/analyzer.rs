use aubio_rs::{OnsetMode, Pitch, PitchMode, Tempo};
use hound::SampleFormat;
use thiserror::Error;

use crate::ui::Dropdown;

/// Represents errors that can occur during audio analysis operations.
#[derive(Error, Debug)]
pub enum AudioError {
    /// Error returned when loading a WAV file fails.
    #[error("File loading failed: {0}")]
    HoundError(#[from] hound::Error),
    /// Error returned when the audio format is not supported.
    #[error("Unsupported audio format")]
    FormatError,
}

#[derive(Clone, Copy, PartialEq)]
pub struct SongType {
    pub hop_size: usize,
    pub buf_size: usize,
    pub tempo_method: OnsetMode,
}
impl SongType {
    pub fn pop() -> Self {
        Self {
            hop_size: 256,
            buf_size: 512,
            tempo_method: OnsetMode::Complex,
        }
    }

    pub fn techno() -> Self {
        Self {
            hop_size: 128,
            buf_size: 256,
            tempo_method: OnsetMode::Complex,
        }
    }

    pub fn techno2() -> Self {
        Self {
            hop_size: 64,
            buf_size: 128,
            tempo_method: OnsetMode::Complex,
        }
    }

    pub fn custom(hop_size: usize, buf_size: usize, tempo_method: OnsetMode) -> Self {
        Self {
            hop_size,
            buf_size,
            tempo_method,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SongTypes {
    Pop,
    Techno,
    Custom(usize, usize, OnsetMode),
}
impl SongTypes {
    pub fn get_song_type(&self) -> SongType {
        match self {
            SongTypes::Pop => SongType::pop(),
            SongTypes::Techno => SongType::techno(),
            SongTypes::Custom(hop_size, buf_size, tempo_method) => {
                SongType::custom(*hop_size, *buf_size, *tempo_method)
            }
        }
    }
}
impl Dropdown<SongTypes> for SongTypes {
    fn get_text(&self) -> &str {
        match self {
            SongTypes::Pop => "Pop",
            SongTypes::Techno => "Techno",
            SongTypes::Custom(_, _, _) => "Custom (advanced)",
        }
    }

    fn get_variants() -> Vec<SongTypes> {
        vec![
            SongTypes::Pop,
            SongTypes::Techno,
            SongTypes::Custom(256, 512, OnsetMode::Complex),
        ]
    }
}

/// Analyzer is responsible for loading audio data and extracting features such as pitch, energy, and beat information.
#[derive(Clone)]
pub struct Analyzer {
    /// The audio samples as normalized floating-point values.
    samples: Vec<f32>,
    /// The sample rate of the audio data (Hz).
    sample_rate: u32,
    /// The hop size (number of samples between successive analysis frames).
    hop_size: usize,
}

impl Analyzer {
    /// Creates a new Analyzer with default parameters.
    ///
    /// # Arguments
    /// * `hop_size` Step size between two consecutive analysis instants.
    ///
    /// # Returns
    ///
    /// A new `Analyzer` instance with empty samples and a default sample rate of 44100 Hz.
    pub fn new(hop_size: usize) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate: 44100,
            hop_size,
        }
    }

    /// Returns the duration of the loaded audio in seconds.
    pub fn duration(&self) -> f32 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.samples.len() as f32 / self.sample_rate as f32
        }
    }

    pub fn get_hop(&self) -> usize {
        self.hop_size
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Loads audio data from a WAV file and returns an Analyzer with the samples and sample rate set.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to the WAV file to load.
    ///
    /// # Errors
    ///
    /// Returns `AudioError::HoundError` if the file cannot be read, or `AudioError::FormatError` if the format is unsupported.
    ///
    /// # Example
    ///
    /// ```rust
    /// let analyzer = Analyzer::new().with_file("input.wav")?;
    /// ```
    pub fn with_file(mut self, path: &str) -> Result<Self, AudioError> {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        self.sample_rate = reader.spec().sample_rate;

        self.samples = match reader.spec().sample_format {
            SampleFormat::Float => reader
                .into_samples::<f32>()
                .collect::<Result<Vec<_>, _>>()?,
            SampleFormat::Int => {
                let max_val = (1i32 << (reader.spec().bits_per_sample - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / max_val))
                    .collect::<Result<Vec<_>, _>>()?
            }
        };

        if spec.channels != 1 {
            self.samples = self.convert_to_mono(spec.channels);
        }

        Ok(self)
    }

    /// Convert samples to mono (single channel) audio.
    fn convert_to_mono(&self, channels: u16) -> Vec<f32> {
        if channels == 1 {
            self.samples.to_vec()
        } else {
            self.samples
                .chunks(channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect()
        }
    }

    /// Returns all the time locations (in seconds) of beats in the song.
    ///
    /// # Arguments
    /// * `buf_size` - Length of FFT
    pub fn extract_beats(&self, buf_size: usize) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut tempo = Tempo::new(
            OnsetMode::SpecDiff,
            buf_size,
            self.hop_size,
            self.sample_rate,
        )?;
        let mut beats = Vec::new();
        for chunk in self.samples.chunks(tempo.get_hop()) {
            if let Ok(beat) = tempo.do_result(chunk) {
                if beat > 0. {
                    let time_in_seconds = tempo.get_last_s();
                    beats.push(time_in_seconds);
                }
            }
        }

        Ok(beats)
    }

    /// Returns a vector of (time, pitch) doubles throughout the song.
    ///
    /// # Arguments
    /// * `buf_size` - Size of the input buffer to analse
    /// * `silence_threshold` - The threshold below which the signal is considered silent (in dB).
    /// * `tolerance` - yin tolerance threshold
    pub fn extract_pitch(
        &self,
        buf_size: usize,
        silence_threshold: f32,
        tolerance: f32,
    ) -> Result<Vec<(f32, f32)>, Box<dyn std::error::Error>> {
        let mut pitch_detector =
            Pitch::new(PitchMode::Yin, buf_size, self.hop_size, self.sample_rate)?;
        pitch_detector.set_silence(silence_threshold);
        pitch_detector.set_tolerance(tolerance);

        let mut timestamps = Vec::new();
        let mut pitches = Vec::new();

        for (i, chunk) in self.samples.chunks(pitch_detector.get_hop()).enumerate() {
            if chunk.len() < self.hop_size {
                break; // skip incomplete chunks
            }

            if let Ok(pitch) = pitch_detector.do_result(chunk) {
                let time_in_seconds =
                    (i * pitch_detector.get_hop()) as f32 / self.sample_rate as f32;
                timestamps.push(time_in_seconds);
                pitches.push(pitch);
            }
        }

        Ok(timestamps.into_iter().zip(pitches).collect())
    }

    /// Extract audio features from the loaded audio samples.
    ///
    /// # Arguments
    /// * `buf_size` - The size of the analysis buffer (in samples).
    /// * `tempo_method` - The algorithm to be used for beat tracking.
    /// * `silence_threshold` - The threshold below which the signal is considered silent (in dB).
    /// * `tolerance` - The tolerance for pitch detection.
    ///
    /// # Returns
    /// * A result containing the extracted features or an error.
    ///
    /// # Errors
    /// * Returns an error if the beat or pitch detection fails.
    pub fn extract_features(
        &self,
        buf_size: usize,
        tempo_method: OnsetMode,
        silence_threshold: f32,
        tolerance: f32,
    ) -> Result<SongFeatures, Box<dyn std::error::Error>> {
        let mut tempo = Tempo::new(tempo_method, buf_size, self.hop_size, self.sample_rate)?;
        let mut pitch_detector =
            Pitch::new(PitchMode::Yin, buf_size, self.hop_size, self.sample_rate)?;
        pitch_detector.set_silence(silence_threshold);
        pitch_detector.set_tolerance(tolerance);

        let mut features = Vec::new();

        for (i, chunk) in self.samples.chunks(self.hop_size).enumerate() {
            if chunk.len() < self.hop_size {
                break; // skip incomplete chunks
            }

            let time_in_seconds = self.calculate_timestamp(i);

            let is_beat = match tempo.do_result(chunk) {
                Ok(beat) => beat > 0.,
                Err(_) => false,
            };

            let pitch = match pitch_detector.do_result(chunk) {
                Ok(p) => p,
                Err(_) => 0.0,
            };

            // let volume = (aubio_rs::db_spl(chunk) + 100.).max(0.);
            let volume = aubio_rs::level_lin(chunk);

            let bpm = if tempo.get_confidence() > 0.1 {
                tempo.get_bpm()
            } else {
                0.0
            };

            features.push(FrameFeatures {
                timestamp: time_in_seconds,
                pitch,
                volume,
                is_beat,
                bpm,
            });
        }

        Ok(SongFeatures(features))
    }

    /// Calculate the time (in seconds) a frame is located in the song from its `frame_index`.
    pub fn calculate_timestamp(&self, frame_index: usize) -> f32 {
        (frame_index * self.hop_size) as f32 / self.sample_rate as f32
    }

    /// Returns (hop_time_ms, frame_duration_ms)
    pub fn calculate_durations(&self, buf_size: usize) -> (f64, f64) {
        let hop_time_ms = (self.hop_size as f64 / self.sample_rate as f64) * 1000.0;
        let frame_duration_ms = (buf_size as f64 / self.sample_rate as f64) * 1000.0;
        (hop_time_ms, frame_duration_ms)
    }
}

#[derive(Clone, Debug)]
pub struct FrameFeatures {
    pub timestamp: f32,
    pub pitch: f32,
    pub volume: f32,
    pub bpm: f32,
    pub is_beat: bool,
}

/// A collection of features extracted from an audio track, organized by frame.
#[derive(Clone)]
pub struct SongFeatures(Vec<FrameFeatures>);
impl SongFeatures {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn inner(&self) -> &Vec<FrameFeatures> {
        &self.0
    }

    pub fn beat_timestamps(&self) -> Vec<f32> {
        self.0.iter().map(|frame| frame.timestamp).collect()
    }

    pub fn attribute_timestamps<F>(&self, attribute_fn: F) -> Vec<(f32, f32)>
    where
        F: Fn(&FrameFeatures) -> f32,
    {
        self.0
            .iter()
            .map(|ff| (ff.timestamp, attribute_fn(ff)))
            .collect()
    }

    pub fn min_attribute<F>(&self, attribute_fn: F) -> f32
    where
        F: Fn(&FrameFeatures) -> f32,
    {
        self.0.iter().map(attribute_fn).fold(f32::MAX, f32::min)
    }

    pub fn max_attribue<F>(&self, attribute_fn: F) -> f32
    where
        F: Fn(&FrameFeatures) -> f32,
    {
        self.0.iter().map(attribute_fn).fold(f32::MIN, f32::max)
    }
}
