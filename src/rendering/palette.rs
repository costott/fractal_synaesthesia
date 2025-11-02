/// A colour palette used for rendering
use macroquad::prelude::*;
use std::collections::HashSet;

/// Linear interpolation between `a` to `b` with parameter `t`.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1f32 - t) * a + t * b
}

/// Linear interpolation between colours `c1` to `c1` with parameter `fraction`.
pub fn interpolate_colour(c1: Color, c2: Color, fraction: f32) -> Color {
    Color::new(
        lerp(c1.r, c2.r, fraction),
        lerp(c1.g, c2.g, fraction),
        lerp(c1.b, c2.b, fraction),
        lerp(c1.a, c2.a, fraction),
    )
}

/// Alpha blending between the current `background` and the `foreground`.
fn alpha_blend(bg: Color, fg: Color) -> Color {
    let out_alpha = fg.a + bg.a * (1.0 - fg.a);

    if out_alpha <= 0.0 {
        return BLANK;
    }

    Color::new(
        (fg.r * fg.a + bg.r * bg.a * (1.0 - fg.a)) / out_alpha,
        (fg.g * fg.a + bg.g * bg.a * (1.0 - fg.a)) / out_alpha,
        (fg.b * fg.a + bg.b * bg.a * (1.0 - fg.a)) / out_alpha,
        out_alpha,
    )
}

/// Alpha blend over the `background` and `foreground` with `strength`.
pub fn blend_colours(bg: Color, fg: Color, strength: f32) -> Color {
    let scaled_fg = fg.with_alpha(fg.a * strength);
    alpha_blend(bg, scaled_fg)
}

/// A colour palette used for assigning a colour to a given [`Layer`](crate::rendering::render_layer::Layer) output value.
#[derive(Clone)]
pub struct Palette {
    pub colour_map: ColourMap,
    pub mapping_type: PaletteMappingType,
    /// The percentage (between `(0.0, 1.0]`) of the palette taken up by 1 repetition.
    length: f32,
    /// The percentage (between `[0.0, 1.0]`) of the palette (proportional to length) offset for the start colour.
    offset: f32,
    /// Stores the previously generated palette.
    pub palette_cache: Vec<Color>,
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
        colours: Vec<Color>,
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
    /// Set the length to `new` and return whether or not the length was changed.
    pub fn set_length(&mut self, new: f32) -> bool {
        assert!(0.0 < new && new <= 1.);
        if self.length == new {
            return false;
        }
        self.length = new;
        true
    }

    pub fn get_offset(&self) -> f32 {
        self.offset
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

    /// Returns the colour at the given `percentage`.
    ///
    /// # Arguments
    ///
    /// * `apply_offset` - whether or not the palette's offset should be taken into consideration.
    /// Most of the time will be `true`, however `false` useful when showing the unedited palette for editing.
    pub fn get_colour_at_percentage(&self, mut percent: f32, apply_offset: bool) -> Color {
        assert!(0.0 <= percent && percent <= 1.0);

        if apply_offset {
            percent += self.offset;
            percent = percent % 1.;
        }

        self.colour_map.get_colour_at_percentage(percent)
    }

    /// Returns a vector of colours for every iteration up to `max_iterations` using the [`PaletteMappingType::Constant`] method.
    fn get_constant_palette(&self, max_iterations: usize) -> Vec<Color> {
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
    fn get_repeated_palette(&self, max_iterations: usize) -> Vec<Color> {
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
    pub fn get_colour_at_layer_output(&self, l_output: f64) -> Color {
        let i = l_output.floor() as usize;
        let next_i = (i + 1).min(self.palette_cache.len() - 1);

        interpolate_colour(
            self.palette_cache[i],
            self.palette_cache[next_i],
            (l_output % 1.0) as f32,
        )
    }

    /// Returns the full gradient as a texture of the required size.
    pub fn get_full_gradient(&self, width: f32, height: f32) -> Texture2D {
        let mut image = Image::gen_image_color(width as u16, height as u16, WHITE);

        for i in 0..width as u32 {
            let colour = self.get_colour_at_percentage(i as f32 / (width - 1.0), false);
            for j in 0..height as u32 {
                image.set_pixel(i, j, colour);
            }
        }

        Texture2D::from_image(&image)
    }

    /// Gets the full palette as a texture of the required size.
    pub fn get_full_palette(&self, width: f32, height: f32) -> Texture2D {
        let mut image = Image::gen_image_color(width as u16, height as u16, WHITE);
        let max_iterations = self.palette_cache.len() - 1;

        for i in 0..width as u32 {
            let iteration = max_iterations as f32 * (i as f32 / (width - 1.));
            let colour = self.get_colour_at_layer_output(iteration as f64);
            for j in 0..height as u32 {
                image.set_pixel(i, j, colour);
            }
        }

        Texture2D::from_image(&image)
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
#[derive(Clone)]
pub struct ColourMap {
    inner: Vec<ColourPoint>,
}
impl ColourMap {
    pub fn new(points: Vec<ColourPoint>) -> Self {
        Self { inner: points }
    }

    /// Creates a new colour map from the given `colours` vectors so each Color is evenly spaced.
    pub fn new_even(colours: Vec<Color>) -> Self {
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

    fn get_colour_at_percentage(&self, percent: f32) -> Color {
        let sorted = self.sort().inner;
        // index of the colour point immediately after the given percentage
        let next_i = sorted
            .iter()
            .position(|p| p.percent_pos > percent)
            .unwrap_or(sorted.len() - 1);

        let (prev, next) = (sorted[next_i - 1], sorted[next_i]);
        interpolate_colour(
            prev.colour,
            next.colour,
            (percent - prev.percent_pos) / (next.percent_pos - prev.percent_pos),
        )
    }
}
impl Default for ColourMap {
    fn default() -> Self {
        Self::new_even(vec![BLACK, WHITE])
    }
}

/// A colour and position (between 0 and 100) in a colour map used for significant colour points
/// in the palette.
#[derive(Clone, Copy)]
pub struct ColourPoint {
    pub colour: Color,
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
            colour: interpolate_colour(point1.colour, point2.colour, percent),
            percent_pos: lerp(point1.percent_pos, point2.percent_pos, percent),
        }
    }
}
impl Into<ColourPoint> for (Color, f32) {
    fn into(self) -> ColourPoint {
        ColourPoint {
            colour: self.0,
            percent_pos: self.1,
        }
    }
}

/// Determines how percentages map to colours for a palette.
#[derive(Clone, Copy)]
pub enum PaletteMappingType {
    /// The palette stays the same regardless of the max iterations.
    ///
    /// Higher max iterations = more iteration points used to represent the same palette.
    Constant,
    /// The palette length stays the same, being extended further with a higher max iterations.
    Repeated,
}
