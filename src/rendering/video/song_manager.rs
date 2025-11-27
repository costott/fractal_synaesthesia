use std::sync::{Arc, Mutex};

use crate::{
    audio::{analyzer::SongFeatures, song::Song},
    rendering::{manager::layer_manager::LayerManager, video::audio_mapper::AudioMapper},
};

/// Loaded song
pub struct SongManager {
    pub song: Song,
    song_featuers: SongFeatures,
    audio_mapping: AudioMapper,
}
impl SongManager {
    pub fn get_layers_at_timestamp(
        &self,
        layer_manger: Arc<Mutex<LayerManager>>,
        timestamp: f32,
    ) -> LayerManager {
        let original_layer_manager = layer_manger.lock().unwrap();
        let mut frame_layers = Vec::with_capacity(original_layer_manager.layers.capacity());

        for (i, layer) in original_layer_manager.layers.iter().enumerate() {
            frame_layers.push(self.audio_mapping.run_on_layer(
                layer,
                i,
                &self.song_featuers,
                timestamp,
            ));
        }

        LayerManager::new(frame_layers, false)
    }

    /// Returns the zoom speed multiplier at the specific timetamp
    /// for use in creating a zoom timeline
    pub fn get_zoom_multiplier_at_timestamp(
        &self,
        layer_manager: Arc<Mutex<LayerManager>>,
        timestamp: f32,
    ) -> f64 {
        let layer_manager = layer_manager.lock().unwrap();
        let mut multiplier = 1.0;

        for (i, _) in layer_manager.layers.iter().enumerate() {
            multiplier += self.audio_mapping.get_zoom_increase_at_timestamp(
                i,
                &self.song_featuers,
                timestamp,
            );
        }

        multiplier
    }
}
