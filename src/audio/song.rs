use macroquad::audio::{self, Sound};

use crate::audio::analyzer::{Analyzer, SongType};

#[derive(Clone)]
pub struct Song {
    sound: Sound,
    song_type: SongType,
    analyzer: Analyzer,
}
impl Song {
    pub async fn load_from_path(
        path: &str,
        song_type: SongType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            analyzer: Analyzer::new(song_type.hop_size).with_file(path)?,
            song_type,
            sound: audio::load_sound(path).await?,
        })
    }

    pub fn duration(&self) -> f32 {
        self.analyzer.duration()
    }
}
