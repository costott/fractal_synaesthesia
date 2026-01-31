use crate::{
    project::Project,
    ui::{
        AppModeScreen,
        fractal::{fractal_canvas::CanvasDimensions, fractal_window::FractalWindow},
        window::WindowParams,
    },
};
use macroquad::prelude::*;

mod controls;
use controls::Controls;
mod param_editor;
mod sidebar;
use sidebar::Sidebar;
mod layers_editor;

pub const START_PIXEL_STEP: f64 = 0.005;

pub struct FractalSettingsMode {
    context: FractalSettingsContext,

    main_fractal: FractalWindow,
    sidebar: Sidebar,
    controls: Controls,
}
impl FractalSettingsMode {
    pub fn new(project: &Project) -> Self {
        Self {
            main_fractal: FractalWindow::new(
                WindowParams {
                    width: 800 as u16,
                    height: 450 as u16,
                    x: (screen_width() - (screen_width() - 500.0 + 800.0) * 0.5) as u16,
                    y: (screen_height() - (screen_height() - 150.0 + 450.0) * 0.5) as u16,
                },
                project.fractal_settings.params.clone(),
                project.fractal_settings.layers.clone(),
            ),
            context: FractalSettingsContext {
                fractal_dims: CanvasDimensions {
                    width: 800,
                    height: 450,
                },
                rendering: false,
                request_render: false,
                update_previews: true,
                update_layers: false,
            },
            sidebar: Sidebar::new(WindowParams {
                width: 500 as u16,
                height: screen_height() as u16 - 150 - 1,
                x: 0,
                y: 150 + 1,
            }),
            controls: Controls::new(WindowParams {
                width: screen_width() as u16,
                height: 150,
                x: 0,
                y: 0,
            }),
        }
    }
}
impl AppModeScreen for FractalSettingsMode {
    fn update(&mut self, project: &mut Project, egui_ctx: &egui::Context) -> bool {
        egui_ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(egui::Color32::DARK_GRAY);
            style.visuals.extreme_bg_color = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.inactive.bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.hovered.bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.active.bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.active.weak_bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.open.bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.widgets.open.weak_bg_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.window_fill = egui::Color32::LIGHT_GRAY;
            style.visuals.selection.bg_fill = crate::ui::ACCENT_COLOUR;
        });

        self.main_fractal
            .update(egui_ctx, project, &mut self.context);
        self.sidebar.update(egui_ctx, project, &mut self.context);
        let save_and_close = self.controls.update(egui_ctx, &mut self.context);

        save_and_close
    }

    fn draw(&self) {
        clear_background(WHITE);

        // background behind main fractal
        draw_rectangle(
            self.sidebar.params.width as f32,
            self.controls.params.height as f32,
            screen_width() - self.sidebar.params.width as f32,
            screen_height() - self.controls.params.height as f32,
            LIGHTGRAY,
        );

        self.main_fractal.draw();
    }
}

#[derive(Clone)]
pub struct FractalSettingsContext {
    pub fractal_dims: CanvasDimensions,
    pub rendering: bool,
    pub request_render: bool,
    pub update_previews: bool,
    pub update_layers: bool,
}

pub trait FractalSettingsWindow {
    fn update(
        &mut self,
        _egui_ctx: &egui::Context,
        _project: &mut Project,
        _ctx: &mut FractalSettingsContext,
    );
}
