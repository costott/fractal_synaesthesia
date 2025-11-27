use crate::{
    audio::song::Song, rendering::video::VideoFrameManager,
    ui::menus::audio_mapper::AudioMapperWindow,
};

pub struct VideoPreviewWindow {}
impl VideoPreviewWindow {}
impl AudioMapperWindow for VideoPreviewWindow {
    fn update(&mut self, _egui_ctx: &egui::Context, _ctx: &mut super::AudioMapperContext) {}
}
