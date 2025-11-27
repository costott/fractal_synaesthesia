use crate::ui::{
    menus::fractal_settings::{
        FractalSettingsWindow, layers_editor::LayersEditor, param_editor::ParamEditor,
    },
    window::WindowParams,
};
use macroquad::prelude::*;

#[derive(PartialEq, Eq)]
enum SidebarTab {
    Parameters,
    Layers,
}
impl SidebarTab {
    fn tabs() -> [SidebarTab; 2] {
        [SidebarTab::Parameters, SidebarTab::Layers]
    }

    fn get_text(&self) -> &str {
        match self {
            SidebarTab::Parameters => "Parameters",
            SidebarTab::Layers => "Layers",
        }
    }
}

pub struct Sidebar {
    pub params: WindowParams,
    is_open: bool,

    selected_tab: SidebarTab,
    param_editor: ParamEditor,
    layers_editor: LayersEditor,
}
impl Sidebar {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            is_open: true,
            selected_tab: SidebarTab::Parameters,
            param_editor: ParamEditor::new(WindowParams {
                width: params.width - 5,
                height: params.height - 50,
                x: params.x + 5,
                y: params.y + 50,
            }),
            layers_editor: LayersEditor::new(WindowParams {
                width: params.width,
                height: params.height - 30,
                x: params.x,
                y: params.y + 30,
            }),
        }
    }
}
impl FractalSettingsWindow for Sidebar {
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
    ) {
        self.params.sized_area("fractalsidebar", egui_ctx, |ui| {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(ui.max_rect().right() + 1.0, ui.max_rect().top()),
                    egui::Pos2::new(ui.max_rect().right() + 1.0, ui.max_rect().bottom()),
                ],
                egui::Stroke::new(2.0, egui::Color32::BLACK),
            );

            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0, 0.0);

                let button_width = ui.available_width() * 0.5;

                for tab in SidebarTab::tabs() {
                    let is_selected = self.selected_tab == tab;
                    let button = egui::Button::new(tab.get_text()).fill(if is_selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::LIGHT_GRAY
                    });

                    if ui.add_sized([button_width, 30.0], button).clicked() {
                        self.selected_tab = tab;
                    }
                }
            });

            match self.selected_tab {
                SidebarTab::Parameters => self.param_editor.update(egui_ctx, ctx),
                SidebarTab::Layers => self.layers_editor.update(egui_ctx, ctx),
            }
        });
    }
}
