use crate::art::draw;
use crate::core::{dims, to_color_image, App, BlendMode, Combine, LineColor};
use egui::{Button, ComboBox, Frame, Grid, Vec2};
use serde_json;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

const SPACE: f32 = 7.0;

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        if let Some(storage) = cc.storage {
            let mut app: App = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if let Some(path) = &app.img_path_1 {
                app.img_1 = image::open(path).unwrap().to_rgba8();
            }
            if let Some(path) = &app.img_path_2 {
                app.img_2 = image::open(path).unwrap().to_rgba8();
            }
            return app;
        }

        Default::default()
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Serialize to JSON
        let json = serde_json::to_string_pretty(self)?;

        // Write to file
        let path = path.with_extension("json");
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    // Load from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Read file contents
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize from JSON
        let app: App = serde_json::from_str(&contents)?;

        Ok(app)
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .pick_file()
                            {
                                *self = App::load_from_file(&path).unwrap();
                                let path1 = self.img_path_1.clone().unwrap();
                                self.img_1 = image::open(path1).unwrap().to_rgba8();
                                let path2 = self.img_path_2.clone().unwrap();
                                self.img_2 = image::open(path2).unwrap().to_rgba8();
                            }
                            ui.close_menu();
                        }
                        if ui.button("Save").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                self.save_to_file(&path).unwrap();
                            }
                            ui.close_menu();
                        }
                        if ui.button("Export").clicked() {
                            let img = draw(&self);
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                let path = path.with_extension("png");
                                img.save(&path).unwrap();
                                println!("Image Saved");
                                println!("-----------------------------");
                            }

                            ui.close_menu();
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
            });
        });

        egui::SidePanel::left("side_panel")
            .exact_width(325.0)
            .resizable(false)
            .frame(Frame::default().inner_margin(15.0))
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.add_space(SPACE);
                if ui
                    .add(
                        Button::new(egui::RichText::new("  Image 1").strong().size(16.0))
                            .min_size(Vec2::new(150.0, 25.0)),
                    )
                    .on_hover_ui(|ui| {
                        ui.colored_label(egui::Color32::ORANGE, "Click to select the file");
                        ui.colored_label(egui::Color32::ORANGE, "path for image 1.");
                    })
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        self.img_path_1 = Some(path.display().to_string());
                        self.img_1 = image::open(&path).unwrap().to_rgba8();
                    }
                }
                ui.add_space(SPACE);
                if let Some(picked_path) = &self.img_path_1 {
                    ui.label(picked_path);
                }
                ui.add_space(SPACE);
                Grid::new("image 1 grid")
                    .spacing((20.0, 10.0))
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.label("Blur").on_hover_ui(|ui| {
                            ui.colored_label(
                                egui::Color32::ORANGE,
                                "Set the standard deviation of",
                            );
                            ui.colored_label(egui::Color32::ORANGE, "the Guassian Blur kernel,");
                            ui.colored_label(egui::Color32::ORANGE, "to apply to image 1.");
                        });
                        ui.add(egui::Slider::new(&mut self.img_blur_1, 0.0..=300.0).step_by(5.0));
                        ui.end_row();

                        ui.label("Hue Roatation").on_hover_ui(|ui| {
                            ui.colored_label(egui::Color32::ORANGE, "Rotate the hue of all colors");
                            ui.colored_label(egui::Color32::ORANGE, "in image 1 by the specified");
                            ui.colored_label(egui::Color32::ORANGE, "number of degrees.");
                        });
                        ui.add(egui::Slider::new(&mut self.hue_rotation_1, 0..=360).step_by(5.0));
                        ui.end_row();
                    });

                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);

                if ui
                    .add(
                        Button::new(egui::RichText::new("  Image 2").strong().size(16.0))
                            .min_size(Vec2::new(150.0, 25.0)),
                    )
                    .on_hover_ui(|ui| {
                        ui.colored_label(egui::Color32::ORANGE, "Click to select the file");
                        ui.colored_label(egui::Color32::ORANGE, "path for image 2.");
                    })
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        self.img_path_2 = Some(path.display().to_string());
                        self.img_2 = image::open(&path).unwrap().to_rgba8();
                    }
                }

                ui.add_space(SPACE);
                if let Some(picked_path) = &self.img_path_2 {
                    ui.label(picked_path);
                }
                ui.add_space(SPACE);

                Grid::new("image 2 grid")
                    .spacing((20.0, 10.0))
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.label("Blur").on_hover_ui(|ui| {
                            ui.colored_label(
                                egui::Color32::ORANGE,
                                "Set the standard deviation of",
                            );
                            ui.colored_label(egui::Color32::ORANGE, "the Guassian Blur kernel,");
                            ui.colored_label(egui::Color32::ORANGE, "to apply to image 2.");
                        });
                        ui.add(egui::Slider::new(&mut self.img_blur_2, 0.0..=300.0).step_by(5.0));
                        ui.end_row();

                        ui.label("Hue Roatation").on_hover_ui(|ui| {
                            ui.colored_label(egui::Color32::ORANGE, "Rotate the hue of all colors");
                            ui.colored_label(egui::Color32::ORANGE, "in image 2 by the specified");
                            ui.colored_label(egui::Color32::ORANGE, "number of degrees.");
                        });
                        ui.add(egui::Slider::new(&mut self.hue_rotation_2, 0..=360).step_by(5.0));
                        ui.end_row();
                    });

                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);

                Grid::new("size grid")
                    .spacing((20.0, 10.0))
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.label("Width").on_hover_ui(|ui| {
                            ui.colored_label(egui::Color32::ORANGE, "Set the width of the output");
                            ui.colored_label(egui::Color32::ORANGE, "image in pixels.");
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.width, 0..=28800));
                            if ui.small_button("↺").clicked() {
                                self.width = App::default().width;
                            }
                        });
                        ui.end_row();

                        ui.label("Height").on_hover_ui(|ui| {
                            ui.colored_label(egui::Color32::ORANGE, "Set the height of the output");
                            ui.colored_label(egui::Color32::ORANGE, "image in pixels.");
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.height, 0..=28800));
                            if ui.small_button("↺").clicked() {
                                self.height = App::default().height;
                            }
                        });
                        ui.end_row();

                        ui.label("Style");
                        ui.horizontal(|ui| {
                            ComboBox::from_label("")
                                .width(150.0)
                                .selected_text(format!("{:?}", self.combine))
                                .show_ui(ui, |ui| {
                                    ui.set_width(150.0);
                                    ui.selectable_value(&mut self.combine, Combine::Blend, "Blend");
                                    ui.selectable_value(
                                        &mut self.combine,
                                        Combine::Divide,
                                        "Divide",
                                    );
                                    ui.selectable_value(&mut self.combine, Combine::Mix, "Mix");
                                    ui.selectable_value(&mut self.combine, Combine::Warp, "Warp");
                                });
                        });
                        ui.end_row();
                    });

                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);

                Grid::new("blend grid")
                    .spacing((20.0, 10.0))
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        if self.combine == Combine::Warp {
                            ui.label("Angle Scale");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.angle_scale, 0.0..=20.0)
                                        .step_by(0.1),
                                );
                                if ui.small_button("↺").clicked() {
                                    self.angle_scale = App::default().angle_scale;
                                }
                            });
                            ui.end_row();

                            ui.label("Angle Factor");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.angle_factor, 0.0..=250.0)
                                        .step_by(5.0),
                                );
                                if ui.small_button("↺").clicked() {
                                    self.angle_factor = App::default().angle_factor;
                                }
                            });
                            ui.end_row();

                            ui.label("Radius Scale");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.radius_scale, 0.0..=20.0)
                                        .step_by(0.05),
                                );
                                if ui.small_button("↺").clicked() {
                                    self.radius_scale = App::default().radius_scale;
                                }
                            });
                            ui.end_row();

                            ui.label("Radius Factor");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.radius_factor, 0.0..=5000.0)
                                        .step_by(50.0),
                                );
                                if ui.small_button("↺").clicked() {
                                    self.radius_factor = App::default().radius_factor;
                                }
                            });
                            ui.end_row();
                        }
                        if self.combine == Combine::Divide || self.combine == Combine::Mix {
                            ui.label("Contamination");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.contamination, 0.0..=2.0)
                                        .step_by(0.05),
                                );
                                if ui.small_button("↺").clicked() {
                                    self.contamination = App::default().contamination;
                                }
                            });
                            ui.end_row();

                            ui.label("Roughness");
                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut self.octaves, 0..=8));
                                if ui.small_button("↺").clicked() {
                                    self.octaves = App::default().octaves;
                                }
                            });
                            ui.end_row();

                            ui.label("Cutoff");
                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut self.cutoff, -1.0..=1.0));
                                if ui.small_button("↺").clicked() {
                                    self.cutoff = App::default().cutoff;
                                }
                            });
                            ui.end_row();
                        }

                        if self.combine == Combine::Blend || self.combine == Combine::Mix {
                            ui.label("Blend Mode");
                            ComboBox::from_label("")
                                .width(150.0)
                                .selected_text(format!("{:?}", self.mode))
                                .show_ui(ui, |ui| {
                                    ui.set_min_width(60.0);
                                    if self.combine != Combine::Mix {
                                        ui.selectable_value(
                                            &mut self.mode,
                                            BlendMode::Screen,
                                            "Screen",
                                        );
                                        ui.selectable_value(
                                            &mut self.mode,
                                            BlendMode::Multiply,
                                            "Multiply",
                                        );
                                        ui.selectable_value(
                                            &mut self.mode,
                                            BlendMode::Darken,
                                            "Darken",
                                        );
                                        ui.selectable_value(
                                            &mut self.mode,
                                            BlendMode::Lighten,
                                            "Lighten",
                                        );
                                        ui.selectable_value(
                                            &mut self.mode,
                                            BlendMode::Difference,
                                            "Difference",
                                        );
                                        ui.selectable_value(
                                            &mut self.mode,
                                            BlendMode::Exclusion,
                                            "Exclusion",
                                        );
                                    }
                                    ui.selectable_value(
                                        &mut self.mode,
                                        BlendMode::Overlay,
                                        "Overlay",
                                    );
                                    ui.selectable_value(&mut self.mode, BlendMode::Dodge, "Dodge");
                                    ui.selectable_value(&mut self.mode, BlendMode::Burn, "Burn");
                                    ui.selectable_value(
                                        &mut self.mode,
                                        BlendMode::HardLight,
                                        "Hard Light",
                                    );
                                    ui.selectable_value(
                                        &mut self.mode,
                                        BlendMode::SoftLight,
                                        "Soft Light",
                                    );
                                });
                            ui.end_row();
                        }
                    });

                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);

                Grid::new("screengrid")
                    .spacing((20.0, 10.0))
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.label("Screen Overlay");
                        ui.add(egui::Checkbox::new(&mut self.screen, ""));
                        ui.end_row();

                        ui.label("Line Spacing");
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.spacing, 0.0..=100.0));
                            if ui.small_button("↺").clicked() {
                                self.spacing = App::default().spacing;
                            }
                        });
                        ui.end_row();

                        let original_spacing = ui.spacing().item_spacing.x;
                        ui.spacing_mut().item_spacing.x = 40.0; // Customize as needed
                        ui.label("Line Color");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.line_color, LineColor::Black, "Black");
                            ui.radio_value(&mut self.line_color, LineColor::White, "White");
                        });
                        ui.spacing_mut().item_spacing.x = original_spacing; // Customize as needed
                        ui.end_row();

                        ui.label("Thickness");
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.thickness, 0.0..=10.0).step_by(0.5));
                            if ui.small_button("↺").clicked() {
                                self.thickness = App::default().thickness;
                            }
                        });
                        ui.end_row();

                        ui.label("Subdivisions");
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.subdivisions, 5..=150).step_by(5.0));
                            if ui.small_button("↺").clicked() {
                                self.subdivisions = App::default().subdivisions;
                            }
                        });
                        ui.end_row();

                        ui.label("Min Opacity");
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.min_opacity, 0.0..=1.0));
                            if ui.small_button("↺").clicked() {
                                self.min_opacity = App::default().min_opacity;
                            }
                        });
                        ui.end_row();

                        ui.label("Max Opacity");
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.max_opacity, 0.0..=1.0));
                            if ui.small_button("↺").clicked() {
                                self.max_opacity = App::default().max_opacity;
                            }
                        });
                        ui.end_row();
                    });

                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);

                ui.vertical_centered({
                    |ui| {
                        if ui
                            .add(Button::new("Draw").min_size(Vec2::new(125.0, 25.0)))
                            .clicked()
                        {
                            let size = dims(self.width as f32, self.height as f32);
                            let img = draw(&self);
                            self.texture = Some(ui.ctx().load_texture(
                                "draw",
                                to_color_image(&img, size.0 as u32, size.1 as u32),
                                Default::default(),
                            ));
                        }
                    }
                });
                ui.add_space(SPACE);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(2.0 * SPACE);
            egui::warn_if_debug_build(ui);
            if let Some(txt) = &self.texture {
                let img_size = txt.size_vec2();
                let size = dims(img_size[0], img_size[1]);
                ui.horizontal(|ui| {
                    ui.add_space(SPACE);
                    ui.add_sized(egui::vec2(size.0, size.1), egui::Image::new(txt));
                });
            }
        });
    }
}
