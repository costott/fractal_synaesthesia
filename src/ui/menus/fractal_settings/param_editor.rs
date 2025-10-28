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

    /// Displays a labeled single-line text input for editing a parameter, and
    /// synchronizes its value with external state via provided getter and setter closures.
    ///
    /// This helper abstracts the common egui boilerplate for numeric or string parameters
    /// that are represented as editable text. It handles focus changes, updates the
    /// underlying value when the field loses focus, and refreshes the displayed text
    /// when the field is not focused.
    ///
    /// # Arguments
    ///
    /// * `ui`: The egui [`Ui`] instance to draw into.
    /// * `label`: The text label to display beside the input box.
    /// * `local_value` A mutable reference to the locally cached string representation
    ///   of the parameter. This value is edited directly by the user.
    /// * `get_value`: A closure returning the current string representation of the
    ///   external parameter, used to refresh `local_value` when the text field is not focused.
    /// * `set_value`: — closure that takes the newly entered string and applies it
    ///   to the external parameter when the text field loses focus.
    ///
    /// # Returns
    ///
    /// Returns `true` if the external parameter was modified and a re-render
    /// should be triggered, otherwise returns `false`.
    fn text_param<FGet, FSet>(
        ui: &mut egui::Ui,
        label: &str,
        local_value: &mut String,
        get_value: FGet,
        set_value: FSet,
    ) -> bool
    where
        FGet: Fn() -> String,
        FSet: Fn(String),
    {
        ui.label(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(crate::ui::NORMAL_TEXT_SIZE)),
        );

        let response = ui.text_edit_singleline(local_value);
        let mut request_render = false;

        // Set render value when sumbitted
        if response.lost_focus() {
            set_value(local_value.clone());
            request_render = true;
        }

        // Update UI value when not being changed by UI
        if !response.has_focus() {
            *local_value = get_value();
        }

        ui.end_row();
        request_render
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
        let mut needs_render = false;

        self.params.sized_area("params", egui_ctx, |ui| {
            // TODO: menu doesn't update during a render

            egui::Grid::new("parameters_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    needs_render |= ParamEditor::text_param(
                        ui,
                        "Center (Re)",
                        &mut self.local_param_copy.center_re,
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

                    needs_render |= ParamEditor::text_param(
                        ui,
                        "Center (Im)",
                        &mut self.local_param_copy.center_im,
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

                    needs_render |= ParamEditor::text_param(
                        ui,
                        "Zoom",
                        &mut self.local_param_copy.zoom,
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

                    needs_render |= ParamEditor::text_param(
                        ui,
                        "Max Iterations",
                        &mut self.local_param_copy.max_iterations,
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
                    let response = ui.add(egui::Slider::new(&mut angle_deg, 0.0..=360.0));
                    if response.changed() {
                        ctx.fractal_params.lock().unwrap().rotation = angle_deg.to_radians();
                        needs_render = true;
                    }
                    ui.end_row();
                });
        });

        // egui::Area::new(egui::Id::new("params"))
        //     .fixed_pos(egui::pos2(self.params.x as f32, self.params.y as f32))
        //     .show(egui_ctx, |ui| {});

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
