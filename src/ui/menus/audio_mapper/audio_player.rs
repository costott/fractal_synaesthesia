use crate::audio::song_player::SongPlayer;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AudioPlayerState {
    Playing,
    Paused,
    Ended,
}

/// Formats a given time (in seconds) into mm:ss
fn format_timestamp(time: f64) -> String {
    let minutes = (time / 60.0).floor() as u32;
    let seconds = (time % 60.0).floor() as u32;
    format!("{:02}:{:02}", minutes, seconds)
}

pub struct AudioPlayer {
    song_player: SongPlayer,
    state: AudioPlayerState,
    slider_width: Option<f32>,

    scrubbing_position: Option<f64>,
}
impl AudioPlayer {
    pub fn new(song_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let song_player = SongPlayer::new(song_path)?;

        Ok(Self {
            song_player,
            state: AudioPlayerState::Paused,
            slider_width: None,
            scrubbing_position: None,
        })
    }

    pub fn set_slider_width(mut self, slider_width: f32) -> Self {
        self.slider_width = Some(slider_width);
        self
    }

    fn add_contents(&mut self, ui: &mut egui::Ui) -> egui::Response {
        match self.state {
            AudioPlayerState::Playing => {
                if ui.button("⏸").clicked() {
                    self.song_player.pause();
                    self.state = AudioPlayerState::Paused;
                }
            }
            AudioPlayerState::Paused => {
                if ui.button("▶").clicked() {
                    self.song_player.resume();
                    self.state = AudioPlayerState::Playing;
                }
            }
            AudioPlayerState::Ended => {
                if ui.button("↺").clicked() {
                    self.song_player.play_from(0.0).unwrap();
                    self.state = AudioPlayerState::Playing;
                }
            }
        }

        if self.song_player.is_ended() {
            self.state = AudioPlayerState::Ended;
        }

        ui.label(format_timestamp(
            if let Some(scrubbing_position) = self.scrubbing_position {
                scrubbing_position
            } else {
                self.song_player.get_position()
            },
        ));

        let mut audio_position = self.song_player.get_position();
        if let Some(slider_width) = self.slider_width {
            ui.spacing_mut().slider_width = slider_width;
        }
        let response = ui.add(
            egui::Slider::new(&mut audio_position, 0.0..=self.song_player.get_duration())
                .show_value(false)
                .step_by(1.0),
        );
        if response.dragged() {
            self.scrubbing_position = Some(audio_position);
        }
        if response.drag_stopped() {
            self.song_player.play_from(audio_position).unwrap();
            if self.state == AudioPlayerState::Paused {
                self.song_player.pause();
            }
            self.scrubbing_position = None;
        }

        ui.label(format!("{}", self.song_player.get_duration_formatted()));

        response
    }
}
impl egui::Widget for &mut AudioPlayer {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut width = 100.0;
        if let Some(slider_width) = self.slider_width {
            width += slider_width;
        } else {
            width += 100.0;
        }

        let response = ui.allocate_ui(egui::vec2(width, 30.0), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                self.add_contents(ui)
            })
        });

        response.inner.inner
    }
}
