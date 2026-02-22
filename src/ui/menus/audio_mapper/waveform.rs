use crate::audio::analyzer::SongFeatures;

pub struct Waveform {
    pub samples: Vec<f32>, // normalized to [-1.0, 1.0]
}

impl Waveform {
    /// Create a waveform from the given `features`, with the `width`
    /// being the number of vertical bars/pixels needed in the UI
    pub fn from_features(features: &SongFeatures, width: usize) -> Self {
        let frames = features.inner();
        if frames.is_empty() {
            return Self { samples: vec![] };
        }

        // extract volumes
        let volumes: Vec<f32> = frames.iter().map(|f| f.volume).collect();

        // normalization
        let max = features.max_attribute(|f| f.volume);
        let normalized: Vec<f32> = volumes.iter().map(|v| v / max).collect();

        // downsample / resample to match target width
        let mut samples = Vec::with_capacity(width);
        for i in 0..width {
            let t = i as f32 / (width - 1).max(1) as f32;
            let idx = (t * (normalized.len() - 1) as f32).round() as usize;
            samples.push(normalized[idx]);
        }

        Self { samples }
    }

    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        start_x: f32,
        height: f32,
        width: f32,
        bar_colour: egui::Color32,
    ) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

        let real_x = start_x;
        let real_rect = egui::Rect::from_min_size(egui::pos2(real_x, rect.min.y), rect.size());

        let num_bars = self.samples.len();
        if num_bars == 0 {
            return;
        }

        let bar_width = rect.width() / num_bars as f32;

        for (i, v) in self.samples.iter().enumerate() {
            let x0 = real_rect.left() + i as f32 * bar_width;
            let x1 = x0 + bar_width;

            let y_center = real_rect.center().y;
            let half_h = (v.abs() * 0.5) * real_rect.height();

            let y0 = y_center - half_h;
            let y1 = y_center + half_h;

            let bar_rect = egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1));

            ui.painter().rect_filled(bar_rect, 0.0, bar_colour);
        }
    }
}

pub struct WaveformSlider<'a> {
    pub position: &'a mut f64,
    pub duration: f64,
    pub width: f32,
    pub height: f32,
    pub waveform: &'a Waveform,
}
impl<'a> egui::Widget for WaveformSlider<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response =
            ui.allocate_response(egui::vec2(self.width, self.height), egui::Sense::drag());
        let rect = response.rect;

        // Handle dragging logic
        if let Some(pointer) = response.interact_pointer_pos() {
            let t = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            *self.position = t as f64 * self.duration;
        }

        // Draw waveform
        self.waveform.ui(
            ui,
            rect.left(),
            rect.height(),
            rect.width(),
            egui::Color32::GRAY,
        );

        // Draw playhead
        let t = *self.position / self.duration;
        let x = rect.left() + t as f32 * rect.width();
        let playhead = egui::Rect::from_center_size(
            egui::pos2(x, rect.center().y),
            egui::vec2(2.0, rect.height() * 0.9),
        );
        ui.painter()
            .rect_filled(playhead, 0.0, egui::Color32::WHITE);

        response
    }
}
