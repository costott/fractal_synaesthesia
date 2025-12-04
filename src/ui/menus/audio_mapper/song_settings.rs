use std::path::PathBuf;

use egui::Widget;

use crate::{
    audio::{analyzer::SongTypes, song::Song},
    rendering::video::song_manager::SongManager,
    ui::{menus::audio_mapper::audio_player::AudioPlayer, window::WindowParams},
};

pub struct SongSettings {
    params: WindowParams,

    old_picked_file: Option<PathBuf>,
    picked_file: Option<PathBuf>,

    song_preview: Option<Song>,
    song_preview_playback: Option<AudioPlayer>,

    song_type: SongTypes,
    // silence_threshold: f32,
    // tolerance: f32,
}
impl SongSettings {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            old_picked_file: None,
            picked_file: None,
            song_preview: None,
            song_preview_playback: None,
            song_type: SongTypes::Pop,
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context, ctx: &mut super::AudioMapperContext) {
        self.params.sized_area("song_settings", egui_ctx, |ui| {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(ui.max_rect().left(), ui.max_rect().bottom()),
                    egui::Pos2::new(ui.max_rect().right(), ui.max_rect().bottom()),
                ],
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Import Song").clicked() {
                        let new_file = rfd::FileDialog::new()
                            .add_filter("Audio", &["wav"])
                            .pick_file();

                        // Only accept new file if one was actually picked
                        if let Some(file_path) = new_file {
                            self.picked_file = Some(file_path);
                        }
                    }

                    ui.label(format!(
                        "File: {:?}",
                        self.picked_file
                            .as_ref()
                            .map_or("None", |p| p.to_str().unwrap_or("..."))
                    ));

                    // New file selected
                    if self.picked_file != self.old_picked_file {
                        self.old_picked_file = self.picked_file.clone();

                        // Load song preview
                        if let Some(audio_file) = &self.picked_file {
                            let path_str = audio_file.to_string_lossy().to_string();
                            let song_type = self.song_type.get_song_type();
                            self.song_preview_playback =
                                Some(AudioPlayer::new(&path_str).unwrap().set_slider_width(600.0));
                            self.song_preview = Some(
                                futures::executor::block_on(Song::load_from_path(
                                    path_str, song_type,
                                ))
                                .unwrap(),
                            );
                        }
                    }

                    if let Some(song_preview) = &self.song_preview {
                        if ui.button("analyze").clicked() {
                            let song_type = self.song_type.get_song_type();
                            ctx.song_manager = Some(
                                SongManager::new(
                                    song_preview.clone(),
                                    song_type.buf_size,
                                    song_type.tempo_method,
                                    // TODO: these are just temporaries
                                    0.0,
                                    10.0,
                                )
                                .unwrap(),
                            );
                            self.song_preview_playback
                                .as_mut()
                                .unwrap()
                                .set_waveform(&ctx.song_manager.as_ref().unwrap().song_featuers);
                        }
                    }
                });

                // Show song preview
                if let Some(preview_playback) = self.song_preview_playback.as_mut() {
                    ui.add(preview_playback);
                }
            });
        });
    }
}
