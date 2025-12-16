use std::sync::{Arc, Mutex};

use crate::{
    audio::{analyzer::SongFeatures, song::Song},
    rendering::{manager::layer_manager::LayerManager, video::audio_mapper::AudioMapper},
};

/// Loaded song
#[derive(Clone)]
pub struct SongManager {
    pub song: Song,
    pub song_features: SongFeatures,
}
impl SongManager {
    pub fn new(
        song: Song,
        buf_size: usize,
        tempo_method: aubio_rs::OnsetMode,
        silence_threshold: f32,
        tolerance: f32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            song_features: song.analyzer.extract_features(
                buf_size,
                tempo_method,
                silence_threshold,
                tolerance,
            )?,
            song,
        })
    }

    pub fn get_layers_at_timestamp(
        &self,
        layer_manger: Arc<Mutex<LayerManager>>,
        audio_mapper: &AudioMapper,
        timestamp: f32,
    ) -> LayerManager {
        let original_layer_manager = layer_manger.lock().unwrap();
        let mut frame_layers = Vec::with_capacity(original_layer_manager.layers.capacity());

        for (i, layer) in original_layer_manager.layers.iter().enumerate() {
            frame_layers.push(audio_mapper.run_on_layer(layer, i, &self.song_features, timestamp));
        }

        LayerManager::new(frame_layers, false)
    }

    /// Returns the zoom speed multiplier at the specific timetamp
    /// for use in creating a zoom timeline
    pub fn get_zoom_multiplier_at_timestamp(
        &self,
        layer_manager: Arc<Mutex<LayerManager>>,
        audio_mapper: &AudioMapper,
        timestamp: f32,
    ) -> f64 {
        let layer_manager = layer_manager.lock().unwrap();
        let mut multiplier = 1.0;

        for (i, _) in layer_manager.layers.iter().enumerate() {
            multiplier +=
                audio_mapper.get_zoom_increase_at_timestamp(i, &self.song_features, timestamp);
        }

        multiplier
    }

    pub fn get_rotation_change_at_timestamp(
        &self,
        layer_manager: Arc<Mutex<LayerManager>>,
        audio_mapper: &AudioMapper,
        timestamp: f32,
    ) -> f64 {
        let layer_manager = layer_manager.lock().unwrap();
        let mut rotation_change = 0.0;

        for (i, _) in layer_manager.layers.iter().enumerate() {
            rotation_change +=
                audio_mapper.get_rotation_increase_at_timestamp(i, &self.song_features, timestamp);
        }

        rotation_change
    }
}
