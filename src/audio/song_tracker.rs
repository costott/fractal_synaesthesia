use std::time::Instant;

use crate::audio::analyzer::Analyzer;

/// Tracks a song during playback
pub struct SongTracker {
    start_time: Option<Instant>,
    sample_rate: u32,
    hop_size: usize,
}

impl SongTracker {
    pub fn new(analyzer: &Analyzer) -> Self {
        Self {
            start_time: None,
            sample_rate: analyzer.get_sample_rate(),
            hop_size: analyzer.get_hop(),
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.start_time = None;
    }

    pub fn is_playing(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn playback_time(&self) -> Option<f32> {
        self.start_time.map(|start| start.elapsed().as_secs_f32())
    }

    pub fn target_frame(&self) -> Option<usize> {
        self.start_time.map(|start| {
            let elapsed = start.elapsed().as_secs_f32();
            (elapsed * self.sample_rate as f32 / self.hop_size as f32).floor() as usize
        })
    }
}
