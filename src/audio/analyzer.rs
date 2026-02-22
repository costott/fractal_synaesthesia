use aubio_rs::{FFT, OnsetMode, Pitch, PitchMode, SpecDesc, SpecShape, Tempo};
use hound::SampleFormat;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Copy)]
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
        let low_granularity =
            self.extract_low_granularity(buf_size, tempo_method, silence_threshold, tolerance)?;
        let mid_granularity = self.extract_mid_granularity()?;

        Ok(SongFeatures::from_low_and_mid(
            low_granularity,
            mid_granularity,
        ))
    }

    /// Extract low gruanularity features (every hop size)
    pub fn extract_low_granularity(
        &self,
        buf_size: usize,
        tempo_method: OnsetMode,
        silence_threshold: f32,
        tolerance: f32,
    ) -> Result<Vec<LowGranularityFeatures>, Box<dyn std::error::Error>> {
        let mut tempo = Tempo::new(tempo_method, buf_size, self.hop_size, self.sample_rate)?;
        let mut pitch_detector =
            Pitch::new(PitchMode::Yin, buf_size, self.hop_size, self.sample_rate)?;
        pitch_detector.set_silence(silence_threshold);
        pitch_detector.set_tolerance(tolerance);

        let mut features = Vec::new();

        let mut fft = FFT::new(buf_size)?;
        let mut spec_centroid_desc = SpecDesc::new(SpecShape::Centroid, buf_size)?;
        let mut spec_spread_desc = SpecDesc::new(SpecShape::Spread, buf_size)?;
        let half_len = fft.get_win() / 2;
        let mut prev_norm = vec![0f32; half_len];

        for i in 0.. {
            let start = i * self.hop_size;
            if start + buf_size > self.samples.len() {
                break;
            }
            let chunk = &self.samples[start..start + buf_size];
            if chunk.len() < self.hop_size {
                break; // skip incomplete chunks
            }

            let time_in_seconds = self.calculate_timestamp(i);

            let is_beat = match tempo.do_result(chunk) {
                Ok(beat) => beat > 0.,
                Err(_) => false,
            };

            let pitch = match pitch_detector.do_result(&chunk[0..self.hop_size]) {
                Ok(p) => p,
                Err(_) => 0.0,
            };

            // let volume = (aubio_rs::db_spl(chunk) + 100.).max(0.);
            let volume = aubio_rs::level_lin(&chunk[0..self.hop_size]);

            // compute spectrum and high-level spectral descriptors
            let (spectrum_buf, centroid_hz, spectral_spread) = self.compute_spectrum_features(
                chunk,
                &mut fft,
                &mut spec_centroid_desc,
                &mut spec_spread_desc,
                buf_size,
            )?;

            let norm = &spectrum_buf[..half_len];

            let spectral_flux = Self::compute_spectral_flux(norm, &mut prev_norm);

            let spectral_flatness = Self::compute_spectral_flatness(norm);

            // band energies
            let (bass_energy, mid_energy, high_energy) =
                Self::compute_band_energies(norm, self.sample_rate, fft.get_win());

            let bpm = if tempo.get_confidence() > 0.1 {
                tempo.get_bpm()
            } else {
                0.0
            };

            let energy = volume;

            features.push(LowGranularityFeatures {
                timestamp: time_in_seconds,
                pitch,
                volume,
                is_beat,
                bpm,
                centroid: centroid_hz,
                spectral_flux,
                bass_energy,
                mid_energy,
                high_energy,
                energy,
                spectral_spread,
                spectral_flatness,
            });
        }

        Ok(features)
    }

    /// Extract mid granularity features (every 3 seconds) that can be used to derive arousal and valence metrics.
    pub fn extract_mid_granularity(
        &self,
    ) -> Result<Vec<MidGranularityFeatures>, Box<dyn std::error::Error>> {
        let mut features = Vec::new();

        // step through self.samples in 3 second windows
        let window_size = (self.sample_rate as f32 * 3.0) as usize;
        let hop_size = window_size / 2; // 50% overlap
        let mut start = 0;
        while start < self.samples.len() {
            let end = (start + window_size).min(self.samples.len());
            let window = &self.samples[start..end];

            let (standard_deviation_energy, median_energy, median_spectral_spread) =
                self.compute_mid_window_stats(window)?;

            // use the center of the mid window as its timestamp to avoid lag
            features.push(MidGranularityFeatures {
                timestamp: self.calculate_timestamp((start + window_size / 2) / self.hop_size),
                standard_deviation_energy,
                median_energy,
                median_spectral_spread,
            });

            start += hop_size;
        }
        Ok(features)
    }

    pub fn calculate_timestamp(&self, frame_index: usize) -> f32 {
        (frame_index * self.hop_size) as f32 / self.sample_rate as f32
    }

    /// Returns (hop_time_ms, frame_duration_ms)
    pub fn calculate_durations(&self, buf_size: usize) -> (f64, f64) {
        let hop_time_ms = (self.hop_size as f64 / self.sample_rate as f64) * 1000.0;
        let frame_duration_ms = (buf_size as f64 / self.sample_rate as f64) * 1000.0;
        (hop_time_ms, frame_duration_ms)
    }

    fn compute_spectrum_features(
        &self,
        chunk: &[f32],
        fft: &mut FFT,
        spec_centroid: &mut SpecDesc,
        spec_spread: &mut SpecDesc,
        buf_size: usize,
    ) -> Result<(Vec<f32>, f32, f32), Box<dyn std::error::Error>> {
        let mut spectrum_buf = vec![0f32; fft.get_win()];
        let _ = fft.do_(chunk, spectrum_buf.as_mut_slice());

        let centroid_bin = match spec_centroid.do_result::<&[f32]>(spectrum_buf.as_ref()) {
            Ok(c) => c,
            Err(_) => 0.0,
        };
        let centroid_hz = centroid_bin * self.sample_rate as f32 / buf_size as f32;

        let spectral_spread = match spec_spread.do_result::<&[f32]>(spectrum_buf.as_ref()) {
            Ok(s) => s,
            Err(_) => 0.0,
        };

        Ok((spectrum_buf, centroid_hz, spectral_spread))
    }

    fn compute_spectral_flux(norm: &[f32], prev_norm: &mut [f32]) -> f32 {
        if norm.is_empty() || prev_norm.len() != norm.len() {
            if prev_norm.len() == norm.len() {
                prev_norm.copy_from_slice(norm);
            }
            return 0.0;
        }

        let mut flux_sum = 0f32;
        for (k, &v) in norm.iter().enumerate() {
            let diff = v - prev_norm[k];
            if diff > 0.0 {
                flux_sum += diff;
            }
        }
        prev_norm.copy_from_slice(norm);
        flux_sum / norm.len() as f32
    }

    fn compute_spectral_flatness(norm: &[f32]) -> f32 {
        if norm.is_empty() {
            return 0.0;
        }
        let eps = 1e-12_f32;
        let mut geo_sum = 0f32;
        let mut arith_sum = 0f32;
        for &v in norm.iter() {
            let v_eps = if v <= 0.0 { eps } else { v };
            geo_sum += v_eps.ln();
            arith_sum += v;
        }
        let geo_mean = (geo_sum / norm.len() as f32).exp();
        let arith_mean = arith_sum / norm.len() as f32;
        if arith_mean > 0.0 {
            geo_mean / arith_mean
        } else {
            0.0
        }
    }

    fn compute_band_energies(norm: &[f32], sample_rate: u32, fft_win: usize) -> (f32, f32, f32) {
        if norm.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let half_len = norm.len();
        let low_max_hz = 250.0_f32; // bass
        let mid_max_hz = 4000.0_f32; // mid

        let bin_hz = sample_rate as f32 / fft_win as f32;
        let low_max_bin = (low_max_hz / bin_hz).min((half_len - 1) as f32) as usize;
        let mid_max_bin = (mid_max_hz / bin_hz).min((half_len - 1) as f32) as usize;

        let mut low_sum = 0f32;
        let mut mid_sum = 0f32;
        let mut high_sum = 0f32;
        let mut low_count = 0usize;
        let mut mid_count = 0usize;
        let mut high_count = 0usize;

        for (k, &v) in norm.iter().enumerate() {
            if k <= low_max_bin {
                low_sum += v;
                low_count += 1;
            } else if k <= mid_max_bin {
                mid_sum += v;
                mid_count += 1;
            } else {
                high_sum += v;
                high_count += 1;
            }
        }

        let bass_energy = if low_count > 0 {
            low_sum / low_count as f32
        } else {
            0.0
        };
        let mid_energy = if mid_count > 0 {
            mid_sum / mid_count as f32
        } else {
            0.0
        };
        let high_energy = if high_count > 0 {
            high_sum / high_count as f32
        } else {
            0.0
        };

        (bass_energy, mid_energy, high_energy)
    }

    fn median(values: &mut [f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = values.len() / 2;
        if values.len() % 2 == 1 {
            values[mid]
        } else {
            (values[mid - 1] + values[mid]) / 2.0
        }
    }

    fn compute_mid_window_stats(
        &self,
        window: &[f32],
    ) -> Result<(f32, f32, f32), Box<dyn std::error::Error>> {
        // collect energy values across hop-sized chunks
        let mut energy_values: Vec<f32> = window
            .chunks(self.hop_size)
            .map(|chunk| aubio_rs::level_lin(chunk))
            .collect();

        // standard deviation (population)
        let standard_deviation_energy = if energy_values.is_empty() {
            0.0
        } else {
            let mean = energy_values.iter().copied().sum::<f32>() / energy_values.len() as f32;
            let var = energy_values
                .iter()
                .map(|x| {
                    let d = *x - mean;
                    d * d
                })
                .sum::<f32>()
                / energy_values.len() as f32;
            var.sqrt()
        };

        let median_energy = {
            if energy_values.is_empty() {
                0.0
            } else {
                Self::median(&mut energy_values)
            }
        };

        // median spectral spread across 512-sample frames
        let mut spectral_spreads = Vec::new();
        let mut fft = FFT::new(512)?;
        let mut specdesc = SpecDesc::new(SpecShape::Spread, 512)?;
        for chunk in window.chunks(512) {
            if chunk.len() < 512 {
                break;
            }
            let mut spectrum_buf = vec![0f32; fft.get_win()];
            let _ = fft.do_(chunk, spectrum_buf.as_mut_slice());
            if let Ok(spread) = specdesc.do_result::<&[f32]>(spectrum_buf.as_ref()) {
                spectral_spreads.push(spread);
            }
        }
        let median_spectral_spread = if spectral_spreads.is_empty() {
            0.0
        } else {
            spectral_spreads.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut tmp = spectral_spreads.clone();
            Self::median(&mut tmp)
        };

        Ok((
            standard_deviation_energy,
            median_energy,
            median_spectral_spread,
        ))
    }
}

/// Features accessible to the user for each frame of the audio for visualisation.
#[derive(Clone, Debug)]
pub struct FrameFeatures {
    pub timestamp: f32,
    pub pitch: f32,
    pub volume: f32,
    pub bpm: f32,
    pub is_beat: bool,
    pub centroid: f32,      // brightness
    pub spectral_flux: f32, // rate of change / activity
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub arousal: f32, // intensity
    pub valence: f32, // positivity
}
impl FrameFeatures {
    pub fn get_feature(&self, feature_type: &FeatureType) -> f32 {
        match feature_type {
            FeatureType::Pitch => self.pitch,
            FeatureType::Volume => self.volume,
            FeatureType::Tempo => self.bpm,
            FeatureType::Beat => self.is_beat as u8 as f32,
            FeatureType::Brightness => self.centroid,
            FeatureType::Activity => self.spectral_flux,
            FeatureType::BassEnergy => self.bass_energy,
            FeatureType::MidEnergy => self.mid_energy,
            FeatureType::HighEnergy => self.high_energy,
            FeatureType::Intensity => self.arousal,
            FeatureType::Positivity => self.valence,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeatureType {
    Pitch,
    Volume,
    Tempo,
    Beat,
    Brightness,
    Activity,
    BassEnergy,
    MidEnergy,
    HighEnergy,
    Intensity,
    Positivity,
}

/// Features computed at the hop size level (e.g. 10ms) that capture low-level audio characteristics.
struct LowGranularityFeatures {
    pub timestamp: f32,
    pub pitch: f32,
    pub volume: f32,
    pub bpm: f32,
    pub is_beat: bool,
    pub centroid: f32,
    pub spectral_flux: f32,
    pub bass_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub energy: f32,
    pub spectral_spread: f32,
    pub spectral_flatness: f32,
}

/// Features computed over mid-level windows (e.g. 3 seconds) that can be used to derive arousal and valence metrics.
struct MidGranularityFeatures {
    pub timestamp: f32,
    pub standard_deviation_energy: f32,
    pub median_energy: f32,
    pub median_spectral_spread: f32,
}

/// A collection of features extracted from an audio track, organized by frame.
#[derive(Clone)]
pub struct SongFeatures(Vec<FrameFeatures>);
impl SongFeatures {
    fn from_low_and_mid(
        low: Vec<LowGranularityFeatures>,
        mid: Vec<MidGranularityFeatures>,
    ) -> Self {
        let mut features: Vec<FrameFeatures> = Vec::with_capacity(low.len());

        let (mid_arousal, mid_valence, mid_timestamps) =
            SongFeatures::compute_mid_arousal_valence(&low, &mid);

        // Interpolate mid metrics to per-frame timestamps.
        let mut mid_idx: usize = 0;
        for lf in low.into_iter() {
            let t = lf.timestamp;
            let arousal = SongFeatures::interpolate_through_mid(
                &mid_arousal,
                &mid_timestamps,
                &mut mid_idx,
                t,
            );
            let valence = SongFeatures::interpolate_through_mid(
                &mid_valence,
                &mid_timestamps,
                &mut mid_idx,
                t,
            );

            features.push(FrameFeatures {
                timestamp: lf.timestamp,
                pitch: lf.pitch,
                volume: lf.volume,
                bpm: lf.bpm,
                is_beat: lf.is_beat,
                centroid: lf.centroid,
                spectral_flux: lf.spectral_flux,
                bass_energy: lf.bass_energy,
                mid_energy: lf.mid_energy,
                high_energy: lf.high_energy,
                arousal,
                valence,
            });
        }

        SongFeatures(features)
    }

    // Compute mid-window arousal and valence vectors along with their timestamps.
    fn compute_mid_arousal_valence(
        low: &[LowGranularityFeatures],
        mid: &[MidGranularityFeatures],
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        const MID_WINDOW_SECONDS: f32 = 3.0;
        let mut mid_arousal = Vec::with_capacity(mid.len());
        let mut mid_valence = Vec::with_capacity(mid.len());
        let mid_timestamps: Vec<f32> = mid.iter().map(|m| m.timestamp).collect();

        for m in mid.iter() {
            // average over low-level features in the mid window
            let start = m.timestamp;
            let end = start + MID_WINDOW_SECONDS;

            let mut sum_energy = 0.0_f32;
            let mut sum_spread = 0.0_f32;
            let mut sum_flatness = 0.0_f32;
            let mut cnt = 0_f32;

            for lf in low.iter() {
                if lf.timestamp >= start && lf.timestamp < end {
                    sum_energy += lf.energy;
                    sum_spread += lf.spectral_spread;
                    sum_flatness += lf.spectral_flatness;
                    cnt += 1.0;
                }
            }

            let avg_energy = if cnt > 0.0 { sum_energy / cnt } else { 0.0 };
            let avg_spread = if cnt > 0.0 { sum_spread / cnt } else { 0.0 };
            let avg_flatness = if cnt > 0.0 { sum_flatness / cnt } else { 0.0 };

            let a = SongFeatures::compute_arousal(
                avg_energy,
                m.standard_deviation_energy,
                m.median_energy,
            );
            let v =
                SongFeatures::compute_valence(avg_spread, m.median_spectral_spread, avg_flatness);
            mid_arousal.push(a);
            mid_valence.push(v);
        }

        (mid_arousal, mid_valence, mid_timestamps)
    }

    fn compute_arousal(energy: f32, std_dev_energy: f32, median_energy: f32) -> f32 {
        (energy + std_dev_energy + median_energy) / 3.0
    }

    fn compute_valence(
        spectral_spread: f32,
        median_spectral_spread: f32,
        spectral_flatness: f32,
    ) -> f32 {
        (spectral_spread + median_spectral_spread + spectral_flatness) / 3.0
    }

    /// Performs linear interpolation of mid-level features to align with low-level feature timestamps.
    fn interpolate_through_mid(
        mid_feature_values: &Vec<f32>,
        timestamps: &Vec<f32>,
        mid_idx: &mut usize,
        t: f32,
    ) -> f32 {
        if mid_feature_values.is_empty() || timestamps.is_empty() {
            return 0.0;
        }

        let last = timestamps.len() - 1;
        if t <= timestamps[0] {
            return mid_feature_values[0];
        }
        if t >= timestamps[last] {
            return mid_feature_values[last];
        }

        while *mid_idx + 1 < timestamps.len() && timestamps[*mid_idx + 1] <= t {
            *mid_idx += 1;
        }

        if *mid_idx + 1 >= timestamps.len() {
            return mid_feature_values[*mid_idx];
        }

        let t0 = timestamps[*mid_idx];
        let t1 = timestamps[*mid_idx + 1];
        let alpha = if (t1 - t0).abs() > f32::EPSILON {
            (t - t0) / (t1 - t0)
        } else {
            0.0
        };

        mid_feature_values[*mid_idx] * (1.0 - alpha) + mid_feature_values[*mid_idx + 1] * alpha
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn inner(&self) -> &Vec<FrameFeatures> {
        &self.0
    }

    pub fn beat_timestamps(&self) -> Vec<f32> {
        self.0
            .iter()
            .filter(|ff| ff.is_beat)
            .map(|frame| frame.timestamp)
            .collect()
    }

    pub fn feature_timestamps(&self, feature_type: &FeatureType) -> Vec<(f32, f32)> {
        self.0
            .iter()
            .map(|ff| (ff.timestamp, ff.get_feature(feature_type)))
            .collect()
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

    pub fn min_feature(&self, feature_type: &FeatureType) -> f32 {
        self.0
            .iter()
            .map(|ff| ff.get_feature(feature_type))
            .fold(f32::MAX, f32::min)
    }

    pub fn max_attribute<F>(&self, attribute_fn: F) -> f32
    where
        F: Fn(&FrameFeatures) -> f32,
    {
        self.0.iter().map(attribute_fn).fold(f32::MIN, f32::max)
    }

    pub fn max_feature(&self, feature_type: &FeatureType) -> f32 {
        self.0
            .iter()
            .map(|ff| ff.get_feature(feature_type))
            .fold(f32::MIN, f32::max)
    }
}
