use crate::{
    types::ComplexNumber,
    ui::{Window, WindowContext, WindowParams},
};

pub struct ParamEditor {
    params: WindowParams,

    is_open: bool,
}
impl ParamEditor {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            is_open: true,
        }
    }
}
impl Window for ParamEditor {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open(&mut self, open: bool) {
        self.is_open = open
    }

    fn update(&mut self, egui_ctx: &egui::Context, ctx: &mut WindowContext) {
        egui::Area::new(egui::Id::new("params"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(egui_ctx, |ui| {
                ui.label(egui::RichText::new("Parameters").font(egui::FontId::proportional(40.0)));

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Center (Re)").font(egui::FontId::proportional(20.0)),
                    );

                    let mut center_re_string = ctx
                        .fractal_params
                        .lock()
                        .unwrap()
                        .center
                        .lock()
                        .unwrap()
                        .real_string();
                    let response = ui.text_edit_singleline(&mut center_re_string);
                    if response.lost_focus()
                        || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        ctx.fractal_params
                            .lock()
                            .unwrap()
                            .center
                            .lock()
                            .unwrap()
                            .update_real_from_string(center_re_string);
                        ctx.request_render = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Rotation").font(egui::FontId::proportional(20.0)),
                    );

                    let mut angle_deg = ctx.fractal_params.lock().unwrap().rotation.to_degrees();
                    let response = ui.add(egui::Slider::new(&mut angle_deg, 0.0..=360.0));
                    if response.changed() {
                        ctx.fractal_params.lock().unwrap().rotation = angle_deg.to_radians();
                        ctx.request_render = true;
                    }
                });
            });
    }
}
