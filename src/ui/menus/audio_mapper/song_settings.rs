use std::{path::PathBuf, sync::mpsc};

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
    /// Whether the song is currently being analysed, used for a spinner
    analysing: bool,
    song_manager_sender: mpsc::Sender<SongManager>,
    song_manager_reciever: mpsc::Receiver<SongManager>,
}
impl SongSettings {
    pub fn new(params: WindowParams) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            params,
            old_picked_file: None,
            picked_file: None,
            song_preview: None,
            song_preview_playback: None,
            // TODO: change song type
            song_type: SongTypes::Pop,
            analysing: false,
            song_manager_sender: tx,
            song_manager_reciever: rx,
        }
    }

    /// Updates the song settings menu
    ///
    /// Returns if a new song manager has been created from analysis
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut super::AudioMapperContext,
    ) -> bool {
        let mut ret = false;

        // recieve sent song manager
        if let Ok(song_manager) = self.song_manager_reciever.try_recv() {
            self.song_preview_playback
                .as_mut()
                .unwrap()
                .set_waveform(&song_manager.song_features);
            ctx.song_manager = Some(song_manager);
            self.analysing = false;
            ret = true;
        }

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
                            .set_directory(dirs::audio_dir().unwrap_or_else(|| ".".into()))
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

                    if self.analysing {
                        ui.spinner();
                    }

                    // New file selected
                    if self.picked_file != self.old_picked_file {
                        self.old_picked_file = self.picked_file.clone();

                        // Load song preview + spawn sender to analyze
                        if let Some(audio_file) = &self.picked_file {
                            let path_str = audio_file.to_string_lossy().to_string();
                            ctx.song_path = Some(path_str.clone());

                            let song_type = self.song_type.get_song_type();
                            self.song_preview_playback =
                                Some(AudioPlayer::new(&path_str).unwrap().set_slider_width(600.0));
                            self.song_preview = Some(
                                futures::executor::block_on(Song::load_from_path(
                                    path_str, song_type,
                                ))
                                .unwrap(),
                            );
                            let tx = self.song_manager_sender.clone();
                            let song_preview = self.song_preview.clone().unwrap();
                            self.analysing = true;
                            std::thread::spawn(move || {
                                tx.send(
                                    SongManager::new(
                                        song_preview,
                                        song_type.buf_size,
                                        song_type.tempo_method,
                                        // TODO: these are just temporaries
                                        -70.0,
                                        0.3,
                                    )
                                    .unwrap(),
                                )
                            });
                        }
                    }
                });

                // Show song preview
                if let Some(preview_playback) = self.song_preview_playback.as_mut() {
                    ui.add(preview_playback);
                }
            });
        });

        ret
    }
}
