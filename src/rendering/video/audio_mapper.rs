use std::collections::HashMap;

use crate::{
    audio::analyzer::{FrameFeatures, SongFeatures},
    rendering::{manager::layer::Layer, palette::alpha_blend},
};

pub struct AudioMapper {
    on_beat: OnBeatMapper,
    tempo: ContinuousSampleMapper,
    volume: ContinuousSampleMapper,
    pitch: ContinuousSampleMapper,
}
impl AudioMapper {
    pub fn empty() -> Self {
        Self {
            on_beat: OnBeatMapper::empty(),
            tempo: ContinuousSampleMapper::empty(),
            volume: ContinuousSampleMapper::empty(),
            pitch: ContinuousSampleMapper::empty(),
        }
    }

    pub fn run_on_layer(
        &self,
        layer: &Layer,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> Layer {
        let mut layer_effect = LayerEffect::none();

        layer_effect.combine_with(&self.on_beat.layer_effect_at(
            layer_index,
            song_features,
            timestamp,
        ));
        for mapper in &[&self.tempo, &self.volume, &self.pitch] {
            layer_effect.combine_with(&mapper.layer_effect_at(
                layer_index,
                song_features,
                timestamp,
            ));
        }

        layer_effect.apply_to_layer(layer)
    }

    pub fn get_zoom_increase_at_timestamp(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64 {
        let mut increase = 0.0;
        increase += self
            .on_beat
            .get_zoom_speed_increase_at(layer_index, song_features, timestamp);
        increase
    }
}

pub trait Mapper {
    fn layer_effect_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> LayerEffect;

    fn get_zoom_speed_increase_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64;
}

pub struct LayerEffect {
    /// Shift a layer's pallete's offset by a certain amoung
    palette_shift: f32,
    /// Make an entire layer's palette flash this colour
    flash_colour: macroquad::color::Color,
}
impl LayerEffect {
    pub fn apply_to_layer(&self, layer: &Layer) -> Layer {
        let mut applied_layer = layer.clone();

        applied_layer.palette.add_offset(self.palette_shift);

        applied_layer
    }

    pub fn none() -> Self {
        Self {
            palette_shift: 0.0,
            flash_colour: macroquad::prelude::BLANK,
        }
    }

    pub fn combine_with(&mut self, other: &Self) {
        self.add_palette_shift(other.palette_shift);
        self.combine_flash_colours(other.flash_colour);
    }

    pub fn add_palette_shift(&mut self, palette_shift: f32) {
        self.palette_shift += palette_shift;
        self.palette_shift = self.palette_shift % 1.0;
    }

    pub fn combine_flash_colours(&mut self, flash_colour: macroquad::color::Color) {
        self.flash_colour = alpha_blend(self.flash_colour, flash_colour);
    }
}

struct OnBeatMapper {
    /// maps layer index to beat action
    layer_actions: HashMap<usize, Vec<OnBeatLayerAction>>,
    zoom_actions: HashMap<usize, OnBeatZoomAction>,

    attack_duration: f32,
    decay_duration: f32,
}
impl OnBeatMapper {
    pub fn empty() -> Self {
        Self {
            layer_actions: HashMap::new(),
            zoom_actions: HashMap::new(),
            attack_duration: 0.1,
            decay_duration: 0.2,
        }
    }

    /// Takes the `timestamp` and `nearest_beat_timestamp` and returns the
    /// intensity of the beat effect relative to where we are to the beat
    fn beat_intensity(&self, timestamp: f32, nearest_beat_timestamp: f32) -> f32 {
        let dt = timestamp - nearest_beat_timestamp;
        if -dt > self.attack_duration || dt > self.decay_duration {
            // outside beat window
            0.0
        } else if dt < 0.0 {
            // in attack phase
            1.0 + dt / self.attack_duration
        } else {
            // in decay phase
            1.0 - dt / self.decay_duration
        }
    }

    /// Returns the beat intensity of the `timetamp` using the nearest beat timestamp from `beat_timestamps`.
    fn intensity_from_nearest_beat(&self, timestamp: f32, beat_timestamps: &[f32]) -> f32 {
        if beat_timestamps.is_empty() {
            return 0.0;
        }

        let nearest = beat_timestamps
            .iter()
            .min_by(|a, b| {
                let da = (timestamp - **a).abs();
                let db = (timestamp - **b).abs();
                da.partial_cmp(&db).unwrap()
            })
            .unwrap();

        self.beat_intensity(timestamp, *nearest)
    }
}
impl Mapper for OnBeatMapper {
    fn layer_effect_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> LayerEffect {
        let mut effect = LayerEffect::none();
        let beat_intensity =
            self.intensity_from_nearest_beat(timestamp, song_features.beat_timestamps().as_slice());

        let action = self.layer_actions.get(&layer_index);
        if let Some(found_action) = action {
            for beat_action in found_action {
                match beat_action {
                    OnBeatLayerAction::Flash(c) => {
                        effect.combine_flash_colours(c.with_alpha(beat_intensity))
                    }
                    OnBeatLayerAction::ShiftPalette(shift) => {
                        effect.add_palette_shift(*shift * beat_intensity)
                    }
                    _ => todo!(),
                }
            }
        }

        effect
    }

    fn get_zoom_speed_increase_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64 {
        let mut increase = 0.0;
        let beat_intensity =
            self.intensity_from_nearest_beat(timestamp, song_features.beat_timestamps().as_slice());

        let action = self.zoom_actions.get(&layer_index);
        if let Some(zoom_action) = action {
            match zoom_action {
                OnBeatZoomAction::AddZoomSpeed(p) => increase += *p * beat_intensity as f64,
                OnBeatZoomAction::Nothing => {}
            }
        }

        increase
    }
}

/// Values are given as maximum values at the peak of the beat
pub enum OnBeatLayerAction {
    Flash(macroquad::color::Color),
    ShiftPalette(f32),
    ShiftPaleteLength(f32),
    Rotate(f64),
    Brighten(f32),
    ShiftHue(f32),
    Saturate(f32),
}

pub enum OnBeatZoomAction {
    /// Add a zoom speed multiplier
    AddZoomSpeed(f64),
    Nothing,
}

/// Mapper for audio features that get continuously sampled (e.g. tempo, volume, pitch)
pub struct ContinuousSampleMapper {
    feature_picker: Box<dyn Fn(&FrameFeatures) -> f32>,
    /// Maps layer index to continous sample action
    layer_actions: HashMap<usize, Vec<ContinousSampleLayerAction>>,
    zoom_actions: HashMap<usize, ContinousSampleZoomAction>,
}
impl ContinuousSampleMapper {
    pub fn empty() -> Self {
        Self {
            feature_picker: Box::new(|_| 0.0),
            layer_actions: HashMap::new(),
            zoom_actions: HashMap::new(),
        }
    }

    fn get_intensity(
        &self,
        timestamp: f32,
        feature_timestamps: &[(f32, f32)],
        max_value: f32,
        min_value: f32,
    ) -> f32 {
        if feature_timestamps.is_empty() {
            return 0.0;
        }

        let (_, nearest_value) = feature_timestamps
            .iter()
            .min_by(|(a, _), (b, _)| {
                let da = (timestamp - *a).abs();
                let db = (timestamp - *b).abs();
                da.partial_cmp(&db).unwrap()
            })
            .unwrap();

        if max_value - min_value == 0.0 {
            0.0
        } else {
            (nearest_value - min_value) / (max_value - min_value)
        }
    }
}
impl Mapper for ContinuousSampleMapper {
    fn layer_effect_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> LayerEffect {
        let mut effect = LayerEffect::none();
        let feature_timestamps = song_features.attribute_timestamps(self.feature_picker.as_ref());
        let feature_timestamps = feature_timestamps.as_slice();

        let min_value = feature_timestamps
            .iter()
            .map(|(_, v)| *v)
            .fold(f32::MAX, f32::min);
        let max_value = feature_timestamps
            .iter()
            .map(|(_, v)| *v)
            .fold(f32::MIN, f32::max);

        let intensity = self.get_intensity(timestamp, feature_timestamps, max_value, min_value);

        if let Some(actions) = self.layer_actions.get(&layer_index) {
            for layer_action in actions {
                match layer_action {
                    ContinousSampleLayerAction::ShiftPalette(shift) => {
                        effect.add_palette_shift(*shift * intensity)
                    }
                    _ => todo!(),
                }
            }
        }

        effect
    }

    fn get_zoom_speed_increase_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64 {
        0.0
    }
}

pub enum ContinousSampleLayerAction {
    ShiftToColour(macroquad::color::Color),
    ShiftPaletteLength(f32),
    ShiftPalette(f32),
    AddRotation(f32),
    HueShift(f32),
    Saturate(f32),
    Brightness(f32),
}

pub enum ContinousSampleZoomAction {
    AddZoomSpeed(f64),
    Nothing,
}
