use crate::types::colour::*;
/// A colour palette used for rendering
use egui::Rgba;
use macroquad::prelude::{Image, Texture2D};
use std::collections::HashSet;

/// Alpha blend over the `background` and `foreground` with `strength`.
pub fn blend_colours(bg: Colour, fg: Colour, strength: f32) -> Colour {
    let scaled_fg = fg.with_alpha(fg.a * strength);
    bg.alpha_blend(&scaled_fg)
}

// pub fn color_to_rbga(color: Color) -> egui::Rgba {
//     fn srgb_to_linear(c: f32) -> f32 {
//         if c <= 0.04045 {
//             c / 12.92
//         } else {
//             ((c + 0.055) / 1.055).powf(2.4)
//         }
//     }

//     egui::Rgba::from_rgba_premultiplied(
//         srgb_to_linear(color.r),
//         srgb_to_linear(color.g),
//         srgb_to_linear(color.b),
//         color.a,
//     )
// }

// pub fn rgba_to_color(rgba: Rgba) -> Color {
//     fn linear_to_srgb(c: f32) -> f32 {
//         if c <= 0.0031308 {
//             c * 12.92
//         } else {
//             1.055 * c.powf(1.0 / 2.4) - 0.055
//         }
//     }

//     Color::new(
//         linear_to_srgb(rgba.r()),
//         linear_to_srgb(rgba.g()),
//         linear_to_srgb(rgba.b()),
//         rgba.a(),
//     )
// }

/// A colour palette used for assigning a colour to a given [`Layer`](crate::rendering::render_layer::Layer) output value.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Palette {
    pub colour_map: ColourMap,
    pub mapping_type: PaletteMappingType,
    /// The percentage (between `(0.0, 1.0]`) of the palette taken up by 1 repetition.
    length: f32,
    /// The percentage (between `[0.0, 1.0]`) of the palette (proportional to length) offset for the start colour.
    offset: f32,
    /// Stores the previously generated palette.
    pub palette_cache: Vec<Colour>,
}
impl Palette {
    /// How many iterations for 1x pallete length for the [`PaletteMappingType::Repeated`] mapping.
    const REPEATED_DEPTH: usize = 500;

    pub fn new(
        colour_map: ColourMap,
        mapping_type: PaletteMappingType,
        length: f32,
        offset: f32,
    ) -> Self {
        assert!(colour_map.has_unique_point_positions());
        assert!(0.0 < length && length <= 1.0);
        assert!(0.0 <= offset && offset <= 1.0);

        Self {
            colour_map,
            mapping_type,
            length,
            offset,
            palette_cache: Vec::new(),
        }
    }

    /// Creates a new colour map with evenly spaced colour points
    pub fn new_even(
        colours: Vec<Colour>,
        mapping_type: PaletteMappingType,
        length: f32,
        offset: f32,
    ) -> Self {
        Self::new(ColourMap::new_even(colours), mapping_type, length, offset)
    }

    pub fn default_shading(mapping_type: PaletteMappingType, length: f32, offset: f32) -> Self {
        Self::new(
            ColourMap::new_even(vec![BLACK, BLANK]),
            mapping_type,
            length,
            offset,
        )
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }
    pub fn get_length_mut(&mut self) -> &mut f32 {
        &mut self.length
    }

    /// Set the length to `new` and return whether or not the length was changed.
    pub fn set_length(&mut self, new: f32) -> bool {
        assert!(0.0 < new && new <= 1.);
        if self.length == new {
            return false;
        }
        self.length = new;
        true
    }

    pub fn multiply_length(&mut self, factor: f32) {
        let new = self.length * factor;
        self.length = new.clamp(f32::MIN_POSITIVE, 1.0);
    }

    pub fn get_offset(&self) -> f32 {
        self.offset
    }
    pub fn get_offset_mut(&mut self) -> &mut f32 {
        &mut self.offset
    }

    /// Set the offset to `new` and return whether or not the offset was changed.
    pub fn set_offset(&mut self, new: f32) -> bool {
        assert!(0.0 <= new && new <= 1.);
        if self.offset == new {
            return false;
        }
        self.offset = new;
        true
    }
    /// Increment the offset by `other`.
    pub fn add_offset(&mut self, other: f32) {
        let new = self.offset + other;
        self.offset = (new + 1.0) % 1.0;
    }

    /// Returns the colour at the given `percentage`.
    ///
    /// # Arguments
    ///
    /// * `apply_offset` - whether or not the palette's offset should be taken into consideration.
    /// Most of the time will be `true`, however `false` useful when showing the unedited palette for editing.
    pub fn get_colour_at_percentage(&self, mut percent: f32, apply_offset: bool) -> Colour {
        assert!(0.0 <= percent && percent <= 1.0);

        if apply_offset {
            percent += self.offset;
            percent = percent % 1.;
        }

        self.colour_map.get_colour_at_percentage(percent)
    }

    /// Returns a vector of colours for every iteration up to `max_iterations` using the [`PaletteMappingType::Constant`] method.
    fn get_constant_palette(&self, max_iterations: usize) -> Vec<Colour> {
        (0..=max_iterations)
            .map(|i| {
                let total_percent = i as f32 / max_iterations as f32;
                // takes the total percent and converts it to a fraction of the palette length
                let length_percent = (total_percent % self.length) / self.length;
                self.get_colour_at_percentage(length_percent, true)
            })
            .collect()
    }

    /// Returns a vector of colours for every iteration up to `max_iterations` using the [`PaletteMappingType::Repeated`] method.
    fn get_repeated_palette(&self, max_iterations: usize) -> Vec<Colour> {
        let colours_per_i = self.length * Self::REPEATED_DEPTH as f32;

        (0..=max_iterations)
            .map(|i| {
                let percent = (i as f32 % colours_per_i) / colours_per_i;
                self.get_colour_at_percentage(percent, true)
            })
            .collect()
    }

    /// Generates and caches the full palette.
    pub fn generate_palette(&mut self, max_iterations: f32) {
        self.palette_cache = match self.mapping_type {
            PaletteMappingType::Constant => self.get_constant_palette(max_iterations as usize),
            PaletteMappingType::Repeated => self.get_repeated_palette(max_iterations as usize),
        };
    }

    /// Returns the colour the pixel that reached the given layer output value `l_output` should be.
    pub fn get_colour_at_layer_output(&self, l_output: f64) -> Colour {
        let i = l_output.floor() as usize;
        let next_i = (i + 1).min(self.palette_cache.len() - 1);

        self.palette_cache[i].interpolate(&self.palette_cache[next_i], (l_output % 1.0) as f32)
    }

    /// Returns the full gradient as a texture of the required size.
    pub fn get_full_gradient(&self, width: f32, height: f32) -> Texture2D {
        let mut image = Image::gen_image_color(width as u16, height as u16, WHITE.into());

        for i in 0..width as u32 {
            let colour = self.get_colour_at_percentage(i as f32 / (width - 1.0), false);
            for j in 0..height as u32 {
                image.set_pixel(i, j, colour.into());
            }
        }

        Texture2D::from_image(&image)
    }

    /// Gets the full palette as a texture of the required size.
    pub fn get_full_palette(&self, width: f32, height: f32) -> Texture2D {
        let mut image = Image::gen_image_color(width as u16, height as u16, WHITE.into());
        let max_iterations = self.palette_cache.len() - 1;

        for i in 0..width as u32 {
            let iteration = max_iterations as f32 * (i as f32 / (width - 1.));
            let colour = self.get_colour_at_layer_output(iteration as f64);
            for j in 0..height as u32 {
                image.set_pixel(i, j, colour.into());
            }
        }

        Texture2D::from_image(&image)
    }

    pub fn add_point(&mut self, percentage: f32) {
        self.colour_map.add_point(percentage);
    }

    pub fn remove_point(&mut self, index: usize) {
        self.colour_map.remove_point(index);
    }

    pub fn apply_flash_colour(&mut self, flash_colour: Colour) {
        for point in self.colour_map.get_map().iter_mut() {
            point.colour = point.colour.alpha_blend(&flash_colour);
        }
    }

    pub fn brighten(&mut self, factor: f32) {
        for point in self.colour_map.get_map().iter_mut() {
            point.colour = Colour::new(
                (point.colour.r * factor).clamp(0.0, 1.0),
                (point.colour.g * factor).clamp(0.0, 1.0),
                (point.colour.b * factor).clamp(0.0, 1.0),
                point.colour.a,
            );
        }
    }

    pub fn shift_hue(&mut self, shift: f32) {
        for point in self.colour_map.get_map().iter_mut() {
            let (mut h, s, l) = macroquad::color::rgb_to_hsl(point.colour.into());
            h = (h + shift) % 1.0;
            point.colour = macroquad::color::hsl_to_rgb(h, s, l).into();
        }
    }

    pub fn saturate(&mut self, factor: f32) {
        for point in self.colour_map.get_map().iter_mut() {
            let (h, mut s, l) = macroquad::color::rgb_to_hsl(point.colour.into());
            s = (s * factor).clamp(0.0, 1.0);
            point.colour = macroquad::color::hsl_to_rgb(h, s, l).into();
        }
    }
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            colour_map: Default::default(),
            mapping_type: PaletteMappingType::Repeated,
            length: 1.0,
            offset: 0.0,
            palette_cache: Vec::new(),
        }
    }
}

/// Map of colour points that outlines the significant colours for a palette.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ColourMap {
    inner: Vec<ColourPoint>,
}
impl ColourMap {
    pub fn new(points: Vec<ColourPoint>) -> Self {
        Self { inner: points }
    }

    /// Creates a new colour map from the given `colours` vectors so each Color is evenly spaced.
    pub fn new_even(colours: Vec<Colour>) -> Self {
        assert!(colours.len() > 1);
        let pos_gap = 1. / (colours.len() - 1) as f32;
        let inner: Vec<ColourPoint> = colours
            .iter()
            .enumerate()
            .map(|(i, c)| (*c, i as f32 * pos_gap).into())
            .collect();

        Self { inner }
    }

    /// Returns whether the colour map has unique point positions for each [`ColourPoint`] inside it.
    fn has_unique_point_positions(&self) -> bool {
        let mut unique = HashSet::new();
        self.inner
            .iter()
            .all(move |c| unique.insert((c.percent_pos * 100.) as u32))
    }

    fn sort(&self) -> Self {
        let mut sorted = self.inner.clone();
        sorted.sort_by_key(|p| (p.percent_pos * 100.) as u32);

        // add the two extremes to both sides so they link together
        let first = sorted.first().unwrap().next_percent();
        let last = sorted.last().unwrap().prev_percent();

        sorted.insert(0, last);
        sorted.push(first);

        Self { inner: sorted }
    }

    pub fn get_colour_at_percentage(&self, percent: f32) -> Colour {
        let sorted = self.sort().inner;
        // index of the colour point immediately after the given percentage
        let next_i = sorted
            .iter()
            .position(|p| p.percent_pos > percent)
            .unwrap_or(sorted.len() - 1);

        let (prev, next) = (sorted[next_i - 1], sorted[next_i]);
        prev.colour.interpolate(
            &next.colour,
            (percent - prev.percent_pos) / (next.percent_pos - prev.percent_pos),
        )
    }

    pub fn get_map(&mut self) -> &mut Vec<ColourPoint> {
        &mut self.inner
    }

    pub fn add_point(&mut self, percent: f32) {
        self.inner.push(ColourPoint {
            percent_pos: percent,
            colour: self.get_colour_at_percentage(percent),
        });
    }

    pub fn remove_point(&mut self, index: usize) {
        if self.inner.len() <= 1 {
            return;
        }

        self.inner.remove(index);
    }
}
impl Default for ColourMap {
    fn default() -> Self {
        Self::new_even(vec![BLACK, WHITE])
    }
}

/// A colour and position (between 0 and 100) in a colour map used for significant colour points
/// in the palette.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ColourPoint {
    pub colour: Colour,
    /// Position of the point in the palette as a percentance
    pub percent_pos: f32,
}
impl ColourPoint {
    /// Returns the colour point of the percentage position directly adjacent to the left of this point.
    fn prev_percent(&self) -> ColourPoint {
        ColourPoint {
            colour: self.colour,
            percent_pos: self.percent_pos - 1.,
        }
    }

    /// Returns the colour point of the percentage position directly adjacent to the right of this point.
    fn next_percent(&self) -> ColourPoint {
        ColourPoint {
            colour: self.colour,
            percent_pos: self.percent_pos + 1.,
        }
    }

    /// [`clamp()`]s the percentage between 0 and 1 to ensure it's valid.
    fn valid_percent(&self) -> f32 {
        self.percent_pos.clamp(0., 1.)
    }

    /// Linear interpolation between points `point1` and `point2` with parameter `percent`.
    fn interpolate_points(point1: &ColourPoint, point2: &ColourPoint, percent: f32) -> ColourPoint {
        ColourPoint {
            colour: point1.colour.interpolate(&point2.colour, percent),
            percent_pos: point1.percent_pos + (point2.percent_pos - point1.percent_pos) * percent,
        }
    }
}
impl Into<ColourPoint> for (Colour, f32) {
    fn into(self) -> ColourPoint {
        ColourPoint {
            colour: self.0,
            percent_pos: self.1,
        }
    }
}

/// Determines how percentages map to colours for a palette.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PaletteMappingType {
    /// The palette stays the same regardless of the max iterations.
    ///
    /// Higher max iterations = more iteration points used to represent the same palette.
    ///
    /// % Iteration -> Colour constant
    Constant,
    /// The palette length stays the same, being repeated further with a higher max iterations.
    ///
    /// Iteration -> Colour constant
    Repeated,
}
impl crate::ui::Dropdown<PaletteMappingType> for PaletteMappingType {
    fn get_variants() -> Vec<PaletteMappingType> {
        [PaletteMappingType::Constant, PaletteMappingType::Repeated].into()
    }

    fn get_text(&self) -> &str {
        match self {
            PaletteMappingType::Constant => "Percentage",
            PaletteMappingType::Repeated => "Iteration",
        }
    }
}
