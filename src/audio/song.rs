use crate::audio::analyzer::{Analyzer, SongType};

#[derive(Clone)]
pub struct Song {
    path: String,
    song_type: SongType,
    pub analyzer: Analyzer,
}
impl Song {
    pub async fn load_from_path(
        path: String,
        song_type: SongType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            analyzer: Analyzer::new(song_type.hop_size).with_file(&path)?,
            path,
            song_type,
        })
    }

    pub fn duration(&self) -> f32 {
        self.analyzer.duration()
    }
}
