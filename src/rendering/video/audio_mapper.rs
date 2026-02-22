use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    audio::analyzer::{FeatureType, SongFeatures},
    rendering::manager::layer::Layer,
    types::colour::*,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioMapper {
    layer_mappers: HashMap<usize, Vec<MapperType>>,
}
impl AudioMapper {
    pub fn empty() -> Self {
        Self {
            layer_mappers: HashMap::new(),
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

        if let Some(mappers) = self.layer_mappers.get(&layer_index) {
            for mapper in mappers {
                layer_effect
                    .combine_with(&mapper.get_mapper().layer_effect(song_features, timestamp));
            }
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

        if let Some(mappers) = self.layer_mappers.get(&layer_index) {
            for mapper in mappers {
                increase += mapper
                    .get_mapper()
                    .get_zoom_speed_increase(song_features, timestamp);
            }
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

        if let Some(mappers) = self.layer_mappers.get(&layer_index) {
            for mapper in mappers {
                increase += mapper
                    .get_mapper()
                    .get_rotation_increase(song_features, timestamp);
            }
        }

        increase
    }

    pub fn layer_mappings_mut(&mut self, layer_index: usize) -> &mut Vec<MapperType> {
        self.layer_mappers
            .entry(layer_index)
            .or_insert_with(Vec::new)
    }
}

pub trait Mapper {
    fn n_mappings(&self) -> usize;

    fn clone_layer_actions(&self) -> Vec<LayerAction>;
    fn set_layer_actions(&mut self, actions: Vec<LayerAction>);

    fn layer_actions_mut(&mut self) -> Vec<&mut LayerAction>;

    fn add_default_mapping(&mut self);
    fn remove_action(&mut self, index: usize) -> Option<LayerAction>;
    fn layer_effect(&self, song_features: &SongFeatures, timestamp: f32) -> LayerEffect;
    fn get_zoom_speed_increase(&self, song_features: &SongFeatures, timestamp: f32) -> f64;
    fn get_rotation_increase(&self, song_features: &SongFeatures, timestamp: f32) -> f64;
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MapperType {
    OnBeat(OnBeatMapper),
    Pitch(ContinuousSampleMapper),
    Volume(ContinuousSampleMapper),
    Tempo(ContinuousSampleMapper),
    Brightness(ContinuousSampleMapper),
    Activity(ContinuousSampleMapper),
    BassEnergy(ContinuousSampleMapper),
    MidEnergy(ContinuousSampleMapper),
    TrebleEnergy(ContinuousSampleMapper),
    Intensity(ContinuousSampleMapper),
    Positivity(ContinuousSampleMapper),
}
impl MapperType {
    pub fn get_mapper(&self) -> &dyn Mapper {
        match self {
            MapperType::OnBeat(m) => m,
            MapperType::Pitch(m)
            | MapperType::Volume(m)
            | MapperType::Tempo(m)
            | MapperType::Brightness(m)
            | MapperType::Activity(m)
            | MapperType::BassEnergy(m)
            | MapperType::MidEnergy(m)
            | MapperType::TrebleEnergy(m)
            | MapperType::Intensity(m)
            | MapperType::Positivity(m) => m,
        }
    }

    pub fn get_mapper_mut(&mut self) -> &mut dyn Mapper {
        match self {
            MapperType::OnBeat(m) => m,
            MapperType::Pitch(m)
            | MapperType::Volume(m)
            | MapperType::Tempo(m)
            | MapperType::Brightness(m)
            | MapperType::Activity(m)
            | MapperType::BassEnergy(m)
            | MapperType::MidEnergy(m)
            | MapperType::TrebleEnergy(m)
            | MapperType::Intensity(m)
            | MapperType::Positivity(m) => m,
        }
    }

    pub fn get_on_beat_mapper_mut(&mut self) -> Option<&mut OnBeatMapper> {
        match self {
            MapperType::OnBeat(m) => Some(m),
            _ => None,
        }
    }

    pub fn get_continuous_sample_mapper_mut(&mut self) -> Option<&mut ContinuousSampleMapper> {
        match self {
            MapperType::Pitch(m)
            | MapperType::Volume(m)
            | MapperType::Tempo(m)
            | MapperType::Brightness(m)
            | MapperType::Activity(m)
            | MapperType::BassEnergy(m)
            | MapperType::MidEnergy(m)
            | MapperType::TrebleEnergy(m)
            | MapperType::Intensity(m)
            | MapperType::Positivity(m) => Some(m),
            _ => None,
        }
    }
}
impl Default for MapperType {
    fn default() -> Self {
        MapperType::OnBeat(OnBeatMapper::empty())
    }
}
impl crate::ui::Dropdown<MapperType> for MapperType {
    fn get_variants() -> Vec<MapperType> {
        vec![
            MapperType::OnBeat(OnBeatMapper::empty()),
            MapperType::Pitch(ContinuousSampleMapper::empty(FeatureType::Pitch)),
            MapperType::Volume(ContinuousSampleMapper::empty(FeatureType::Volume)),
            MapperType::Tempo(ContinuousSampleMapper::empty(FeatureType::Tempo)),
            MapperType::Brightness(ContinuousSampleMapper::empty(FeatureType::Brightness)),
            MapperType::Activity(ContinuousSampleMapper::empty(FeatureType::Activity)),
            MapperType::BassEnergy(ContinuousSampleMapper::empty(FeatureType::BassEnergy)),
            MapperType::MidEnergy(ContinuousSampleMapper::empty(FeatureType::MidEnergy)),
            MapperType::TrebleEnergy(ContinuousSampleMapper::empty(FeatureType::HighEnergy)),
            MapperType::Intensity(ContinuousSampleMapper::empty(FeatureType::Intensity)),
            MapperType::Positivity(ContinuousSampleMapper::empty(FeatureType::Positivity)),
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            MapperType::OnBeat(_) => "On Beat",
            MapperType::Pitch(_) => "Pitch",
            MapperType::Volume(_) => "Volume",
            MapperType::Tempo(_) => "Tempo",
            MapperType::Brightness(_) => "Brightness",
            MapperType::Activity(_) => "Activity",
            MapperType::BassEnergy(_) => "Bass Energy",
            MapperType::MidEnergy(_) => "Mid Energy",
            MapperType::TrebleEnergy(_) => "Treble Energy",
            MapperType::Intensity(_) => "Intensity",
            MapperType::Positivity(_) => "Positivity",
        }
    }
}
impl PartialEq for MapperType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Effect to apply to a layer
pub struct LayerEffect {
    /// Make an entire layer's palette flash this colour
    flash_colour: Colour,
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
            flash_colour: BLANK,
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

    pub fn combine_flash_colours(&mut self, flash_colour: Colour) {
        self.flash_colour = self.flash_colour.alpha_blend(&flash_colour);
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
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum LayerAction {
    AddZoomSpeed(f64),
    Flash(Colour),
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
            LayerAction::Flash(BLANK),
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
                // let mut rgba = crate::rendering::palette::color_to_rbga((*c).into());
                let mut rgba = (*c).into();
                if egui::color_picker::color_edit_button_rgba(
                    ui,
                    &mut rgba,
                    egui::color_picker::Alpha::OnlyBlend,
                )
                .changed()
                {
                    // *c = crate::rendering::palette::rgba_to_color(rgba).into();
                    *c = rgba.into();
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

#[derive(Clone, Serialize, Deserialize)]
pub struct OnBeatMapper {
    /// maps layer index to beat action
    // layer_actions: HashMap<usize, Vec<LayerAction>>,
    layer_actions: Vec<LayerAction>,

    pub attack_duration: f32,
    pub decay_duration: f32,
}
impl OnBeatMapper {
    pub fn empty() -> Self {
        Self {
            layer_actions: Vec::new(),
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
    fn n_mappings(&self) -> usize {
        self.layer_actions.len()
    }

    fn clone_layer_actions(&self) -> Vec<LayerAction> {
        self.layer_actions.clone()
    }

    fn set_layer_actions(&mut self, actions: Vec<LayerAction>) {
        self.layer_actions = actions;
    }

    fn layer_actions_mut(&mut self) -> Vec<&mut LayerAction> {
        self.layer_actions.iter_mut().collect()
    }

    fn add_default_mapping(&mut self) {
        self.layer_actions.push(LayerAction::default());
    }

    fn remove_action(&mut self, index: usize) -> Option<LayerAction> {
        if index < self.layer_actions.len() {
            Some(self.layer_actions.remove(index))
        } else {
            None
        }
    }

    fn layer_effect(&self, song_features: &SongFeatures, timestamp: f32) -> LayerEffect {
        let mut effect = LayerEffect::none();
        let beat_intensity =
            self.intensity_from_nearest_beat(timestamp, song_features.beat_timestamps().as_slice());

        for beat_action in &self.layer_actions {
            effect.apply_action(beat_action, beat_intensity);
        }

        effect
    }

    fn get_zoom_speed_increase(&self, song_features: &SongFeatures, timestamp: f32) -> f64 {
        let mut increase = 0.0;
        let beat_intensity =
            self.intensity_from_nearest_beat(timestamp, song_features.beat_timestamps().as_slice());

        for action in &self.layer_actions {
            match action {
                LayerAction::AddZoomSpeed(p) => increase += *p * beat_intensity as f64,
                _ => {}
            }
        }

        increase
    }

    fn get_rotation_increase(&self, song_features: &SongFeatures, timestamp: f32) -> f64 {
        let mut increase = 0.0;

        let beat_intensity =
            self.intensity_from_nearest_beat(timestamp, song_features.beat_timestamps().as_slice());

        for action in &self.layer_actions {
            match action {
                LayerAction::Rotate(deg) => increase += deg.to_radians() * beat_intensity as f64,
                _ => {}
            }
        }

        increase
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum IntensityEmphasis {
    Equal,     // Linear mapping
    High(u32), // Higher intensities emphasised more (exponential)
    Low(u32),  // Lower intensities emphasised more (inverse exponential)
}
impl IntensityEmphasis {
    pub fn interpolate(&self, current: f32, max: f32, min: f32) -> f32 {
        if max - min == 0.0 {
            return 0.0;
        }

        let fraction = ((current - min) / (max - min)).clamp(0.0, 1.0);
        match self {
            IntensityEmphasis::Equal => fraction,
            IntensityEmphasis::High(exp) => fraction.powi(*exp as i32),
            IntensityEmphasis::Low(exp) => 1.0 - (1.0 - fraction).powi(*exp as i32),
        }
    }
}
impl crate::ui::Dropdown<IntensityEmphasis> for IntensityEmphasis {
    fn get_variants() -> Vec<IntensityEmphasis> {
        vec![
            IntensityEmphasis::Equal,
            IntensityEmphasis::High(1),
            IntensityEmphasis::Low(1),
        ]
    }

    fn get_text(&self) -> &str {
        match self {
            IntensityEmphasis::Equal => "Equal",
            IntensityEmphasis::High(_) => "Highs",
            IntensityEmphasis::Low(_) => "Lows",
        }
    }

    fn get_tooltip(&self) -> Option<String> {
        Some(
            match self {
                IntensityEmphasis::Equal => "Intensity has a linear effect on the mapping",
                IntensityEmphasis::High(_) => {
                    "Higher intensities are emphasised more, with an exponential curve"
                }
                IntensityEmphasis::Low(_) => {
                    "Lower intensities are emphasised more, with an inverse exponential curve"
                }
            }
            .to_string(),
        )
    }
}
impl PartialEq for IntensityEmphasis {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
pub struct ContinuousSampleMapper {
    // feature_picker: Box<dyn Fn(&FrameFeatures) -> f32>,
    feature_type: FeatureType,

    pub window: ContinuousSampleWindow,
    pub intensity_emphasis: IntensityEmphasis,
    /// Maps layer index to layer action
    // layer_actions: HashMap<usize, Vec<LayerAction>>,
    layer_actions: Vec<LayerAction>,
}
impl ContinuousSampleMapper {
    pub fn empty(feature_type: FeatureType) -> Self {
        Self {
            feature_type,
            window: ContinuousSampleWindow {
                duration: 0.1,
                window_type: ContinuousSampleWindowType::Centered,
            },
            intensity_emphasis: IntensityEmphasis::Equal,
            layer_actions: Vec::new(),
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

        self.intensity_emphasis
            .interpolate(average_value, max_value, min_value)
    }

    fn intensity_from_song_features(&self, timestamp: f32, song_features: &SongFeatures) -> f32 {
        let feature_timestamps = song_features.feature_timestamps(&self.feature_type);
        let feature_timestamps = feature_timestamps.as_slice();

        let min_value = song_features.min_feature(&self.feature_type);
        let max_value = song_features.max_feature(&self.feature_type);

        self.get_intensity(timestamp, feature_timestamps, max_value, min_value)
    }
}
impl Mapper for ContinuousSampleMapper {
    fn n_mappings(&self) -> usize {
        self.layer_actions.len()
    }

    fn clone_layer_actions(&self) -> Vec<LayerAction> {
        self.layer_actions.clone()
    }

    fn set_layer_actions(&mut self, actions: Vec<LayerAction>) {
        self.layer_actions = actions;
    }

    fn layer_actions_mut(&mut self) -> Vec<&mut LayerAction> {
        self.layer_actions.iter_mut().collect()
    }

    fn add_default_mapping(&mut self) {
        self.layer_actions.push(LayerAction::default());
    }

    fn remove_action(&mut self, index: usize) -> Option<LayerAction> {
        if index < self.layer_actions.len() {
            Some(self.layer_actions.remove(index))
        } else {
            None
        }
    }

    fn layer_effect(&self, song_features: &SongFeatures, timestamp: f32) -> LayerEffect {
        let mut effect = LayerEffect::none();

        let intensity = self.intensity_from_song_features(timestamp, song_features);

        for action in &self.layer_actions {
            effect.apply_action(action, intensity);
        }

        effect
    }

    fn get_zoom_speed_increase(&self, song_features: &SongFeatures, timestamp: f32) -> f64 {
        let mut increase = 0.0;

        let intensity = self.intensity_from_song_features(timestamp, song_features);

        for action in &self.layer_actions {
            match action {
                LayerAction::AddZoomSpeed(p) => increase += *p * intensity as f64,
                _ => {}
            }
        }

        increase
    }

    fn get_rotation_increase(&self, song_features: &SongFeatures, timestamp: f32) -> f64 {
        let mut increase = 0.0;

        let intensity = self.intensity_from_song_features(timestamp, song_features);

        for action in &self.layer_actions {
            match action {
                LayerAction::Rotate(deg) => increase += deg.to_radians() * intensity as f64,
                _ => {}
            }
        }

        increase
    }
}
