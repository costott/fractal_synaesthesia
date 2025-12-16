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
            tempo: ContinuousSampleMapper::empty(Box::new(|f| f.bpm)),
            volume: ContinuousSampleMapper::empty(Box::new(|f| f.volume)),
            pitch: ContinuousSampleMapper::empty(Box::new(|f| f.pitch)),
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
            tempo: ContinuousSampleMapper::empty(Box::new(|f| f.bpm)),
            volume: ContinuousSampleMapper {
                feature_picker: Box::new(|features: &FrameFeatures| features.volume),
                window: ContinuousSampleWindow {
                    duration: 1.0,
                    window_type: ContinuousSampleWindowType::Centered,
                },
                layer_actions: volume_layer_actions,
            },
            pitch: ContinuousSampleMapper::empty(Box::new(|f| f.pitch)),
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

    pub fn get_rotation_increase_at_timestamp(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64 {
        let mut increase = 0.0;
        increase += self
            .on_beat
            .get_rotation_increase_at(layer_index, song_features, timestamp);
        for mapper in &[&self.tempo, &self.volume, &self.pitch] {
            increase += mapper.get_rotation_increase_at(layer_index, song_features, timestamp);
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
    fn get_rotation_increase_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64;
}

/// Effect to apply to a layer
pub struct LayerEffect {
    /// Make an entire layer's palette flash this colour
    flash_colour: macroquad::color::Color,
    /// Shift a layer's pallete's offset by a certain amoung
    palette_shift: f32,
    palette_length_multiplier: f32,
    brighten_factor: f32,
    hue_shift: f32,
    saturate_factor: f32,
}
impl LayerEffect {
    pub fn apply_to_layer(&self, layer: &Layer) -> Layer {
        let mut applied_layer = layer.clone();

        applied_layer.palette.add_offset(self.palette_shift);
        applied_layer.palette.apply_flash_colour(self.flash_colour);
        applied_layer
            .palette
            .multiply_length(self.palette_length_multiplier + 1.0);
        applied_layer.palette.brighten(self.brighten_factor + 1.0);
        applied_layer.palette.shift_hue(self.hue_shift);
        applied_layer.palette.saturate(self.saturate_factor + 1.0);

        applied_layer
    }

    pub fn none() -> Self {
        Self {
            palette_shift: 0.0,
            flash_colour: macroquad::prelude::BLANK,
            palette_length_multiplier: 0.0,
            brighten_factor: 0.0,
            hue_shift: 0.0,
            saturate_factor: 0.0,
        }
    }

    pub fn combine_with(&mut self, other: &Self) {
        self.combine_flash_colours(other.flash_colour);
        self.add_palette_shift(other.palette_shift);
        self.add_palette_length_multiplier(other.palette_length_multiplier);
        self.add_brighten_factor(other.brighten_factor);
        self.add_hue_shift(other.hue_shift);
        self.add_saturate_factor(other.saturate_factor);
    }

    pub fn combine_flash_colours(&mut self, flash_colour: macroquad::color::Color) {
        self.flash_colour = alpha_blend(self.flash_colour, flash_colour);
    }

    pub fn add_palette_shift(&mut self, palette_shift: f32) {
        self.palette_shift += palette_shift;
    }

    pub fn add_palette_length_multiplier(&mut self, length_multiplier: f32) {
        self.palette_length_multiplier += length_multiplier;
    }

    pub fn add_brighten_factor(&mut self, brighten_factor: f32) {
        self.brighten_factor += brighten_factor;
    }

    pub fn add_hue_shift(&mut self, hue_shift: f32) {
        self.hue_shift += hue_shift;
    }

    pub fn add_saturate_factor(&mut self, saturate_factor: f32) {
        self.saturate_factor += saturate_factor;
    }

    pub fn apply_action(&mut self, action: &LayerAction, intensity: f32) {
        match action {
            LayerAction::AddZoomSpeed(_) => {} // Handled elsewhere
            LayerAction::Rotate(_) => {}       // Handled elsewhere
            LayerAction::Flash(c) => {
                self.combine_flash_colours(c.with_alpha(c.a * intensity));
            }
            LayerAction::ShiftPalette(shift) => {
                self.add_palette_shift(*shift * intensity);
            }
            LayerAction::ChangePaletteLength(factor) => {
                self.add_palette_length_multiplier(*factor * intensity);
            }
            LayerAction::Brighten(factor) => {
                self.add_brighten_factor(*factor * intensity);
            }
            LayerAction::ShiftHue(shift) => {
                self.add_hue_shift(*shift * intensity);
            }
            LayerAction::Saturate(factor) => {
                self.add_saturate_factor(*factor * intensity);
            }
        }
    }
}

/// Values are given as maximum values at the peak of the intensity (i.e. intensity = 1.0)
#[derive(Clone, Copy, PartialEq)]
pub enum LayerAction {
    AddZoomSpeed(f64),
    Flash(macroquad::color::Color),
    ShiftPalette(f32),
    ChangePaletteLength(f32),
    Rotate(f64),
    Brighten(f32),
    ShiftHue(f32),
    Saturate(f32),
}
impl LayerAction {
    pub fn timeline_needs_recompute(&self) -> bool {
        match self {
            LayerAction::AddZoomSpeed(_) => true,
            LayerAction::Rotate(_) => true,
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
            LayerAction::ChangePaletteLength(0.0),
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
            LayerAction::ChangePaletteLength(_) => "Change Palette Length",
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
                    "Multiplier of additional zoom speed to add at maximum intensity"
                }
                LayerAction::Flash(_) => {
                    "Flash the palette of the layer with this colour at maximum intensity"
                }
                LayerAction::ShiftPalette(_) => {
                    "Shift the palette offset of the layer by this amount at maximum intensity"
                }
                LayerAction::ChangePaletteLength(_) => {
                    "Increase the palette length of the layer by this factor at maximum intensity"
                }
                LayerAction::Rotate(_) => {
                    "Rotate the video by this amount (in degrees) at maximum intensity"
                }
                LayerAction::Brighten(_) => {
                    "Increase the brightness of the palette of the layer by this factor at maximum intensity"
                }
                LayerAction::ShiftHue(_) => {
                    "Shift the hue of the layer's palette by this amount at maximum intensity"
                }
                LayerAction::Saturate(_) => {
                    "Increase the saturation of the palette of the layer by this factor at maximum intensity"
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
                let response = ui.add(egui::Slider::new(shift, -1.0..=1.0));
                response.changed()
            }
            LayerAction::ChangePaletteLength(factor) => {
                let response = ui.add(
                    egui::DragValue::new(factor)
                        .speed(0.01)
                        .range(-f32::MAX..=f32::MAX),
                );
                response.changed()
            }
            LayerAction::Rotate(angle) => {
                let response = ui.add(egui::Slider::new(angle, -360.0..=360.0));
                response.changed()
            }
            LayerAction::Brighten(factor) => {
                let response = ui.add(
                    egui::DragValue::new(factor)
                        .speed(0.1)
                        .range(-f32::MAX..=f32::MAX),
                );
                response.changed()
            }
            LayerAction::ShiftHue(shift) => {
                let response = ui.add(egui::Slider::new(shift, 0.0..=1.0));
                response.changed()
            }
            LayerAction::Saturate(factor) => {
                let response = ui.add(
                    egui::DragValue::new(factor)
                        .speed(0.1)
                        .range(-f32::MAX..=f32::MAX),
                );
                response.changed()
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

        if let Some(actions) = self.layer_actions.get(&layer_index) {
            for beat_action in actions {
                effect.apply_action(beat_action, beat_intensity);
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

        if let Some(found_actions) = self.layer_actions.get(&layer_index) {
            for action in found_actions {
                match action {
                    LayerAction::AddZoomSpeed(p) => increase += *p * beat_intensity as f64,
                    _ => {}
                }
            }
        }

        increase
    }

    fn get_rotation_increase_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64 {
        let mut increase = 0.0;

        let beat_intensity =
            self.intensity_from_nearest_beat(timestamp, song_features.beat_timestamps().as_slice());

        if let Some(found_actions) = self.layer_actions.get(&layer_index) {
            for action in found_actions {
                match action {
                    LayerAction::Rotate(deg) => {
                        increase += deg.to_radians() * beat_intensity as f64
                    }
                    _ => {}
                }
            }
        }

        increase
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    pub fn empty(feature_picker: Box<dyn Fn(&FrameFeatures) -> f32>) -> Self {
        Self {
            feature_picker,
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

    fn intensity_from_song_features(&self, timestamp: f32, song_features: &SongFeatures) -> f32 {
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

        self.get_intensity(timestamp, feature_timestamps, max_value, min_value)
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

        let intensity = self.intensity_from_song_features(timestamp, song_features);

        if let Some(actions) = self.layer_actions.get(&layer_index) {
            for layer_action in actions {
                effect.apply_action(layer_action, intensity);
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

        let intensity = self.intensity_from_song_features(timestamp, song_features);

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

    fn get_rotation_increase_at(
        &self,
        layer_index: usize,
        song_features: &SongFeatures,
        timestamp: f32,
    ) -> f64 {
        let mut increase = 0.0;

        let intensity = self.intensity_from_song_features(timestamp, song_features);

        if let Some(actions) = self.layer_actions.get(&layer_index) {
            for layer_action in actions {
                match layer_action {
                    LayerAction::Rotate(deg) => increase += deg.to_radians() * intensity as f64,
                    _ => {}
                }
            }
        }

        increase
    }
}
