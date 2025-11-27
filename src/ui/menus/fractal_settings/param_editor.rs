use crate::{
    types::ComplexNumber,
    ui::{WindowParams, menus::fractal_settings::FractalSettingsWindow},
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
impl FractalSettingsWindow for ParamEditor {
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        ctx: &mut crate::ui::menus::fractal_settings::FractalSettingsContext,
    ) {
        let mut needs_render = false;

        self.params.sized_area("params", egui_ctx, |ui| {
            egui::Grid::new("parameters_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    needs_render |= crate::ui::text_param(
                        ui,
                        "Center (Re)",
                        &mut self.local_param_copy.center_re,
                        ctx.rendering,
                        || {
                            ctx.fractal_params
                                .lock()
                                .unwrap()
                                .center
                                .lock()
                                .unwrap()
                                .real_string()
                        },
                        |v_string| {
                            ctx.fractal_params
                                .lock()
                                .unwrap()
                                .center
                                .lock()
                                .unwrap()
                                .update_real_from_string(v_string);
                        },
                    );

                    needs_render |= crate::ui::text_param(
                        ui,
                        "Center (Im)",
                        &mut self.local_param_copy.center_im,
                        ctx.rendering,
                        || {
                            ctx.fractal_params
                                .lock()
                                .unwrap()
                                .center
                                .lock()
                                .unwrap()
                                .im_string()
                        },
                        |v_string| {
                            ctx.fractal_params
                                .lock()
                                .unwrap()
                                .center
                                .lock()
                                .unwrap()
                                .update_im_from_string(v_string);
                        },
                    );

                    needs_render |= crate::ui::text_param(
                        ui,
                        "Zoom",
                        &mut self.local_param_copy.zoom,
                        ctx.rendering,
                        || {
                            (crate::ui::menus::fractal_settings::START_PIXEL_STEP
                                / ctx.fractal_params.lock().unwrap().pixel_step)
                                .to_string()
                        },
                        |v_string| {
                            if let Ok(val) = v_string.parse::<f64>() {
                                ctx.fractal_params.lock().unwrap().pixel_step =
                                    crate::ui::menus::fractal_settings::START_PIXEL_STEP / val;
                            }
                        },
                    );

                    needs_render |= crate::ui::text_param(
                        ui,
                        "Max Iterations",
                        &mut self.local_param_copy.max_iterations,
                        ctx.rendering,
                        || {
                            ctx.fractal_params
                                .lock()
                                .unwrap()
                                .max_iterations
                                .to_string()
                        },
                        |v_string| {
                            if let Ok(val) = v_string.parse::<u32>() {
                                ctx.fractal_params.lock().unwrap().max_iterations = val;
                            }
                        },
                    );

                    ui.label(
                        egui::RichText::new("Rotation")
                            .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
                    );

                    let mut angle_deg = ctx.fractal_params.lock().unwrap().rotation.to_degrees();
                    let response = ui.add(egui::Slider::new(&mut angle_deg, -180.0..=180.0));
                    if response.changed() {
                        ctx.fractal_params.lock().unwrap().rotation = angle_deg.to_radians();
                        needs_render = true;
                    }
                    ui.end_row();
                });
        });

        if needs_render {
            ctx.request_render = true;
        }
    }
}

struct LocalParamCopy {
    center_re: String,
    center_im: String,
    zoom: String,
    max_iterations: String,
}
impl LocalParamCopy {
    fn new() -> Self {
        Self {
            center_re: "".to_string(),
            center_im: "".to_string(),
            zoom: "".to_string(),
            max_iterations: "".to_string(),
        }
    }
}
