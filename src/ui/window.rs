use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct WindowParams {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
}
impl WindowParams {
    pub fn get_bounding_rect(&self) -> Rect {
        Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }

    pub fn sized_area(
        &self,
        source: impl std::hash::Hash,
        egui_ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        egui::Area::new(egui::Id::new(source))
            .fixed_pos(egui::pos2(self.x as f32, self.y as f32))
            .show(egui_ctx, |ui| {
                ui.set_min_size(egui::vec2(self.width as f32, self.height as f32));
                ui.set_max_size(egui::vec2(self.width as f32, self.height as f32));

                add_contents(ui);
            });
    }
}

/// Minimize the native application window (best-effort). On Windows this
/// triggers the OS minimize; on other platforms this is a no-op.
pub fn minimize_app() {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, SW_MINIMIZE, ShowWindow,
        };

        unsafe {
            let hwnd = GetForegroundWindow();
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        }
    }

    #[cfg(not(windows))]
    {
        // No portable API available here; do nothing.
    }
}

/// Close the native application window (best-effort). On Windows this posts
/// a WM_CLOSE to the foreground window; on other platforms we fall back to
/// exiting the process.
pub fn close_app() {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, PostMessageW, WM_CLOSE,
        };

        unsafe {
            let hwnd = GetForegroundWindow();
            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    #[cfg(not(windows))]
    {
        std::process::exit(0);
    }
}
