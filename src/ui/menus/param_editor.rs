use crate::{
    types::ComplexNumber,
    ui::{Window, WindowContext, WindowParams},
};

pub struct ParamEditor {
    params: WindowParams,

    local_param_copy: LocalParamCopy,
    is_open: bool,
}
impl ParamEditor {
    pub fn new(params: WindowParams) -> Self {
        Self {
            params,
            local_param_copy: LocalParamCopy::new(),
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
                        egui::RichText::new("Center (Re)")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    let response = ui.text_edit_singleline(&mut self.local_param_copy.center_re);
                    if response.lost_focus()
                        || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        ctx.fractal_params
                            .lock()
                            .unwrap()
                            .center
                            .lock()
                            .unwrap()
                            .update_real_from_string(self.local_param_copy.center_re.clone());
                        ctx.request_render = true;
                        response.surrender_focus();
                    }
                    if !response.has_focus() {
                        self.local_param_copy.center_re = ctx
                            .fractal_params
                            .lock()
                            .unwrap()
                            .center
                            .lock()
                            .unwrap()
                            .real_string();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Center (Im)")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    let response = ui.text_edit_singleline(&mut self.local_param_copy.center_im);
                    if response.lost_focus()
                        || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        ctx.fractal_params
                            .lock()
                            .unwrap()
                            .center
                            .lock()
                            .unwrap()
                            .update_im_from_string(self.local_param_copy.center_im.clone());
                        ctx.request_render = true;
                        response.surrender_focus();
                    }
                    if !response.has_focus() {
                        self.local_param_copy.center_im = ctx
                            .fractal_params
                            .lock()
                            .unwrap()
                            .center
                            .lock()
                            .unwrap()
                            .im_string();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Max Iterations")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    let response = ui.text_edit_singleline(&mut self.local_param_copy.max_iterations);
                    if response.lost_focus()
                        || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        if let Ok(val) = self.local_param_copy.max_iterations.parse::<u32>() {
                            ctx.fractal_params.lock().unwrap().max_iterations = val;
                        }
                        ctx.request_render = true;
                        response.surrender_focus();
                    }
                    if !response.has_focus() {
                        self.local_param_copy.max_iterations = ctx.fractal_params.lock().unwrap().max_iterations.to_string();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Rotation")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
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

struct LocalParamCopy {
    center_re: String,
    center_im: String,
    pixel_step: String,
    max_iterations: String,
}
impl LocalParamCopy {
    fn new() -> Self {
        Self {
            center_re: "".to_string(),
            center_im: "".to_string(),
            pixel_step: "".to_string(),
            max_iterations: "".to_string(),
        }
    }
}
