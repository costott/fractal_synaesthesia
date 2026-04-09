use crate::{
    project::Project,
    ui::{
        AppModeScreen, AppSignal,
        fractal::{
            fractal_canvas::CanvasDimensions, fractal_window::FractalWindow,
            zoom_window::ZoomWindowInteractState,
        },
        window::WindowParams,
    },
};
use macroquad::prelude::*;

mod controls;
use controls::Controls;
mod param_editor;
mod sidebar;
use sidebar::Sidebar;
mod fractal_settings_footer;
use fractal_settings_footer::FractalSettingsFooter;
mod layers_editor;

pub const START_PIXEL_STEP: f64 = 0.005;

pub struct FractalSettingsMode {
    context: FractalSettingsContext,

    main_fractal: FractalWindow,
    sidebar: Sidebar,
    controls: Controls,
    footer: FractalSettingsFooter,
}
impl FractalSettingsMode {
    pub fn new(project: &Project) -> Self {
        let controls_height = 50;
        let sidebar_width = 500;
        let footer_height = 30;

        Self {
            main_fractal: FractalWindow::new(
                WindowParams {
                    width: 800 as u16,
                    height: 450 as u16,
                    x: (screen_width() - (screen_width() - sidebar_width as f32 + 800.0) * 0.5)
                        as u16,
                    y: (screen_height() - (screen_height() - controls_height as f32 + 450.0) * 0.5)
                        as u16,
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
                zoom_window_state: ZoomWindowInteractState::Inactive { has_history: false },
            },
            sidebar: Sidebar::new(WindowParams {
                width: sidebar_width,
                height: screen_height() as u16 - controls_height - 1 - footer_height,
                x: 0,
                y: controls_height + 1,
            }),
            controls: Controls::new(WindowParams {
                width: screen_width() as u16,
                height: controls_height,
                x: 0,
                y: 0,
            }),
            footer: FractalSettingsFooter::new(WindowParams {
                width: screen_width() as u16,
                height: footer_height,
                x: 0,
                y: screen_height() as u16 - footer_height,
            }),
        }
    }

    pub fn changed_project(&mut self, project: &Project) {
        self.main_fractal.changed_project(project);
        self.context.request_render = true;
    }
}
impl AppModeScreen for FractalSettingsMode {
    fn update(&mut self, project: &mut Project, egui_ctx: &egui::Context) -> AppSignal {
        egui_ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(egui::Color32::DARK_GRAY);
            style.visuals.widgets.active.fg_stroke =
                egui::Stroke::new(2.0, egui::Color32::from_gray(50));
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
        self.footer.update(egui_ctx, project, &mut self.context);

        if save_and_close {
            AppSignal::SwapMode
        } else {
            AppSignal::None
        }
    }

    fn draw(&self) {
        clear_background(WHITE);

        // background behind main fractal
        draw_rectangle(
            self.sidebar.params.width as f32,
            self.controls.params.height as f32,
            screen_width() - self.sidebar.params.width as f32,
            screen_height() - self.controls.params.height as f32 - self.footer.params.height as f32,
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
    pub zoom_window_state: ZoomWindowInteractState,
}

pub trait FractalSettingsWindow {
    fn update(
        &mut self,
        _egui_ctx: &egui::Context,
        _project: &mut Project,
        _ctx: &mut FractalSettingsContext,
    );
}
