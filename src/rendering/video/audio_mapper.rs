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

    pub fn test() -> Self {
        let on_beat_layer_actions = HashMap::from([(
            0,
            vec![
                // OnBeatLayerAction::Flash(macroquad::prelude::WHITE),
                LayerAction::ShiftPalette(0.25),
            ],
        )]);

        let volume_layer_actions = HashMap::from([(0, vec![LayerAction::AddZoomSpeed(100.0)])]);

        Self {
            on_beat: OnBeatMapper {
                layer_actions: on_beat_layer_actions,
                attack_duration: 0.1,
                decay_duration: 0.1,
            },
            tempo: ContinuousSampleMapper::empty(),
            volume: ContinuousSampleMapper {
                feature_picker: Box::new(|features: &FrameFeatures| features.volume),
                window: ContinuousSampleWindow {
                    duration: 1.0,
                    window_type: ContinuousSampleWindowType::Centered,
                },
                layer_actions: volume_layer_actions,
            },
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
        for mapper in &[&self.tempo, &self.volume, &self.pitch] {
            increase += mapper.get_zoom_speed_increase_at(layer_index, song_features, timestamp);
        }
        increase
    }

    pub fn on_beat_layer_actions_mut(&mut self) -> &mut HashMap<usize, Vec<LayerAction>> {
        &mut self.on_beat.layer_actions
    }

    pub fn tempo_layer_actions_mut(&mut self) -> &mut HashMap<usize, Vec<LayerAction>> {
        &mut self.tempo.layer_actions
    }

    pub fn volume_layer_actions_mut(&mut self) -> &mut HashMap<usize, Vec<LayerAction>> {
        &mut self.volume.layer_actions
    }

    pub fn pitch_layer_actions_mut(&mut self) -> &mut HashMap<usize, Vec<LayerAction>> {
        &mut self.pitch.layer_actions
    }

    pub fn on_beat_attack_duration_mut(&mut self) -> &mut f32 {
        &mut self.on_beat.attack_duration
    }

    pub fn on_beat_decay_duration_mut(&mut self) -> &mut f32 {
        &mut self.on_beat.decay_duration
    }

    pub fn tempo_window_mut(&mut self) -> &mut ContinuousSampleWindow {
        &mut self.tempo.window
    }

    pub fn volume_window_mut(&mut self) -> &mut ContinuousSampleWindow {
        &mut self.volume.window
    }

    pub fn pitch_window_mut(&mut self) -> &mut ContinuousSampleWindow {
        &mut self.pitch.window
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

/// Effect to apply to a layer
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
        applied_layer.palette.apply_flash_colour(self.flash_colour);

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

/// Values are given as maximum values at the peak of the intensity (i.e. intensity = 1.0)
#[derive(Clone, Copy, PartialEq)]
pub enum LayerAction {
    AddZoomSpeed(f64),
    Flash(macroquad::color::Color),
    ShiftPalette(f32),
    ShiftPaletteLength(f32),
    Rotate(f64),
    Brighten(f32),
    ShiftHue(f32),
    Saturate(f32),
}
impl LayerAction {
    pub fn changes_zoom_timeline(&self) -> bool {
        match self {
            LayerAction::AddZoomSpeed(_) => true,
            _ => false,
        }
    }
}
impl Default for LayerAction {
    fn default() -> Self {
        LayerAction::AddZoomSpeed(0.0)
    }
}
impl crate::ui::Dropdown<LayerAction> for LayerAction {
    fn get_variants() -> Vec<LayerAction> {
        vec![
            LayerAction::AddZoomSpeed(0.0),
            LayerAction::Flash(macroquad::prelude::BLANK),
            LayerAction::ShiftPalette(0.0),
            LayerAction::ShiftPaletteLength(0.0),
            LayerAction::Rotate(0.0),
            LayerAction::Brighten(0.0),
            LayerAction::ShiftHue(0.0),
            LayerAction::Saturate(0.0),
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            LayerAction::AddZoomSpeed(_) => "Add Zoom Speed",
            LayerAction::Flash(_) => "Flash Colour",
            LayerAction::ShiftPalette(_) => "Shift Palette",
            LayerAction::ShiftPaletteLength(_) => "Shift Palette Length",
            LayerAction::Rotate(_) => "Rotate",
            LayerAction::Brighten(_) => "Brighten",
            LayerAction::ShiftHue(_) => "Shift Hue",
            LayerAction::Saturate(_) => "Saturate",
        }
    }

    fn get_tooltip(&self) -> Option<String> {
        Some(
            match self {
                LayerAction::AddZoomSpeed(_) => {
                    "Increase the zoom speed of the video by this multiplier at maximum intensity"
                }
                LayerAction::Flash(_) => {
                    "Flash the palette of the layer with this colour at maximum intensity"
                }
                LayerAction::ShiftPalette(_) => {
                    "Shift the palette offset of the layer by this amount at maximum intensity"
                }
                LayerAction::ShiftPaletteLength(_) => {
                    "Shift the palette length of the layer by this amount at maximum intensity"
                }
                LayerAction::Rotate(_) => {
                    "Rotate the video by this amount (in radians) at maximum intensity"
                }
                LayerAction::Brighten(_) => {
                    "Brighten the palette of the layer by this amount at maximum intensity"
                }
                LayerAction::ShiftHue(_) => {
                    "Shift the hue of the layer's palette by this amount at maximum intensity"
                }
                LayerAction::Saturate(_) => {
                    "Saturate the palette of the layer by this amount at maximum intensity"
                }
            }
            .to_string(),
        )
    }
}
impl crate::ui::menus::audio_mapper::audio_mapping_window::MappingAction for LayerAction {
    fn edit_inner_value(&mut self, ui: &mut egui::Ui) -> bool {
        match self {
            LayerAction::AddZoomSpeed(p) => {
                let response = ui.add(egui::DragValue::new(p).speed(0.1).range(0.0..=f64::MAX));
                response.changed()
            }
            LayerAction::Flash(c) => {
                let mut rgba = crate::rendering::palette::color_to_rbga(*c);
                if egui::color_picker::color_edit_button_rgba(
                    ui,
                    &mut rgba,
                    egui::color_picker::Alpha::OnlyBlend,
                )
                .changed()
                {
                    *c = crate::rendering::palette::rgba_to_color(rgba);
                    true
                } else {
                    false
                }
            }
            LayerAction::ShiftPalette(shift) => {
                let response = ui.add(egui::Slider::new(shift, 0.0..=1.0));
                response.changed()
            }
            _ => {
                ui.label("Editing not implemented yet");
                false
            }
        }
    }
}

struct OnBeatMapper {
    /// maps layer index to beat action
    layer_actions: HashMap<usize, Vec<LayerAction>>,

    attack_duration: f32,
    decay_duration: f32,
}
impl OnBeatMapper {
    pub fn empty() -> Self {
        Self {
            layer_actions: HashMap::new(),
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
                    LayerAction::AddZoomSpeed(_) => {} // Handled elsewhere
                    LayerAction::Flash(c) => {
                        effect.combine_flash_colours(c.with_alpha(beat_intensity))
                    }
                    LayerAction::ShiftPalette(shift) => {
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

        let actions = self.layer_actions.get(&layer_index);
        if let Some(found_actions) = actions {
            for action in found_actions {
                match action {
                    LayerAction::AddZoomSpeed(p) => increase += *p * beat_intensity as f64,
                    _ => {}
                }
            }
        }

        increase
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContinuousSampleWindowType {
    /// Use samples to the left of the timestamp
    Left,
    /// Use samples around the timestamp
    Centered,
    /// Use samples to the right of the timestamp
    Right,
}
impl crate::ui::Dropdown<ContinuousSampleWindowType> for ContinuousSampleWindowType {
    fn get_variants() -> Vec<ContinuousSampleWindowType> {
        vec![
            ContinuousSampleWindowType::Left,
            ContinuousSampleWindowType::Centered,
            ContinuousSampleWindowType::Right,
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            ContinuousSampleWindowType::Left => "Left",
            ContinuousSampleWindowType::Centered => "Centered",
            ContinuousSampleWindowType::Right => "Right",
        }
    }

    fn get_tooltip(&self) -> Option<String> {
        Some(
            match self {
                ContinuousSampleWindowType::Left => "Use samples to the left of the timestamp",
                ContinuousSampleWindowType::Centered => "Use samples around the timestamp",
                ContinuousSampleWindowType::Right => "Use samples to the right of the timestamp",
            }
            .to_string(),
        )
    }
}

pub struct ContinuousSampleWindow {
    pub duration: f32,
    pub window_type: ContinuousSampleWindowType,
}
impl ContinuousSampleWindow {
    pub fn sample_in_window(&self, timestamp: f32, sample_timestamp: f32) -> bool {
        match self.window_type {
            ContinuousSampleWindowType::Left => {
                sample_timestamp >= timestamp - self.duration && sample_timestamp <= timestamp
            }
            ContinuousSampleWindowType::Centered => {
                sample_timestamp >= timestamp - self.duration / 2.0
                    && sample_timestamp <= timestamp + self.duration / 2.0
            }
            ContinuousSampleWindowType::Right => {
                sample_timestamp >= timestamp && sample_timestamp <= timestamp + self.duration
            }
        }
    }
}

/// Mapper for audio features that get continuously sampled (e.g. tempo, volume, pitch)
pub struct ContinuousSampleMapper {
    feature_picker: Box<dyn Fn(&FrameFeatures) -> f32>,

    window: ContinuousSampleWindow,
    /// Maps layer index to layer action
    layer_actions: HashMap<usize, Vec<LayerAction>>,
}
impl ContinuousSampleMapper {
    pub fn empty() -> Self {
        Self {
            feature_picker: Box::new(|_| 0.0),
            window: ContinuousSampleWindow {
                duration: 0.1,
                window_type: ContinuousSampleWindowType::Centered,
            },
            layer_actions: HashMap::new(),
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

        let mut values_in_window = Vec::new();

        for (t, v) in feature_timestamps {
            if self.window.sample_in_window(timestamp, *t) {
                values_in_window.push(*v);
            }
        }

        if values_in_window.is_empty() {
            return 0.0;
        }

        let average_value: f32 =
            values_in_window.iter().sum::<f32>() / (values_in_window.len() as f32);

        if max_value - min_value == 0.0 {
            0.0
        } else {
            (average_value - min_value) / (max_value - min_value)
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
                    LayerAction::AddZoomSpeed(_) => {} // Handled elsewhere
                    LayerAction::ShiftPalette(shift) => {
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
        let mut increase = 0.0;

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
                    LayerAction::AddZoomSpeed(p) => increase += *p * intensity as f64,
                    _ => {}
                }
            }
        }

        increase
    }
}
