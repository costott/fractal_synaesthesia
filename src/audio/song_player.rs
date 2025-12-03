use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend,
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
};

/// A simple song player using the Kira audio library.
pub struct SongPlayer {
    manager: AudioManager,
    loaded_sound: StaticSoundData,
    handle: Option<StaticSoundHandle>,
}
impl SongPlayer {
    pub fn new(song_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
        let sound_data = StaticSoundData::from_file(song_path)?;

        Ok(Self {
            manager,
            loaded_sound: sound_data,
            handle: None,
        })
    }

    /// Play the song from a specific timestamp (in seconds).
    pub fn play_from(&mut self, timestamp: f64) -> Result<(), Box<dyn std::error::Error>> {
        // Stop any currently playing sound
        if let Some(handle) = self.handle.as_mut() {
            handle.stop(Default::default());
        }

        // Play the sound from the specified timestamp
        let mut handle = self.manager.play(self.loaded_sound.clone())?;
        handle.seek_to(timestamp);
        self.handle = Some(handle);

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            handle.pause(Default::default());
        }
    }

    pub fn is_playing(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.state().is_advancing())
            .unwrap_or(false)
    }

    pub fn resume(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            handle.resume(Default::default());
        } else {
            // If there's no handle, start playing from the beginning
            let _ = self.play_from(0.0);
        }
    }

    /// Get the current playback position in seconds.
    ///
    /// Returns 0.0 if no song is currently playing.
    pub fn get_position(&self) -> f64 {
        self.handle.as_ref().map(|h| h.position()).unwrap_or(0.0)
    }

    pub fn get_position_formatted(&self) -> String {
        let pos = self.get_position();
        let minutes = (pos / 60.0).floor() as u32;
        let seconds = (pos % 60.0).floor() as u32;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Get the total duration of the loaded song in seconds.
    pub fn get_duration(&self) -> f64 {
        self.loaded_sound.duration().as_secs_f64()
    }

    pub fn get_duration_formatted(&self) -> String {
        let dur = self.get_duration();
        let minutes = (dur / 60.0).floor() as u32;
        let seconds = (dur % 60.0).floor() as u32;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn is_ended(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.state() == kira::sound::PlaybackState::Stopped)
            .unwrap_or(false)
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(handle) = self.handle.as_mut() {
            handle.set_volume(volume, Default::default());
        }
    }
}
