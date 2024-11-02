use crate::art::draw;
use crate::core::{dims, to_color_image, App};
use directories::UserDirs;
use egui::Grid;
use egui::{Button, Frame, Vec2};
use std::path::PathBuf;

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
                        if ui.button("Export PNG").clicked() {
                            let img = draw(&self);
                            let dirs = UserDirs::new().unwrap();
                            let dir = dirs.download_dir().unwrap();
                            let path = format!(r"{}/{}", dir.to_string_lossy(), "screen");
                            let mut num = 0;
                            let mut sketch = PathBuf::from(format!(r"{path}_{num}"));
                            sketch.set_extension("png");
                            while sketch.exists() {
                                num += 1;
                                sketch = PathBuf::from(format!(r"{path}_{num}"));
                                sketch.set_extension("png");
                            }
                            img.save(sketch).unwrap();
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
            .exact_width(300.0)
            .resizable(false)
            .frame(Frame::default().inner_margin(10.0))
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.add_space(SPACE);
                if ui
                    .add(Button::new("Image 1 Path").min_size(Vec2::new(125.0, 25.0)))
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
                ui.separator();
                ui.add_space(SPACE);
                if ui
                    .add(Button::new("Image 2 Path").min_size(Vec2::new(125.0, 25.0)))
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
                ui.separator();
                ui.add_space(SPACE);

                Grid::new("grid").spacing((20.0, 10.0)).show(ui, |ui| {
                    ui.label("Width");
                    ui.add(egui::Slider::new(&mut self.width, 0..=8064));
                    ui.end_row();

                    ui.label("Height");
                    ui.add(egui::Slider::new(&mut self.height, 0..=8064));
                    ui.end_row();

                    ui.label("Blur 1");
                    ui.add(egui::Slider::new(&mut self.img_blur_1, 0.0..=200.0).step_by(5.0));
                    ui.end_row();

                    ui.label("Blur 2");
                    ui.add(egui::Slider::new(&mut self.img_blur_2, 0.0..=200.0).step_by(5.0));
                    ui.end_row();

                    ui.label("Contamination");
                    ui.add(egui::Slider::new(&mut self.contamination, 0.0..=2.0));
                    ui.end_row();

                    ui.label("Roughness");
                    ui.add(egui::Slider::new(&mut self.octaves, 1..=8));
                    ui.end_row();

                    ui.label("Cutoff");
                    ui.add(egui::Slider::new(&mut self.cutoff, -1.0..=1.0));
                    ui.end_row();
                });

                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);

                Grid::new("grid").spacing((20.0, 10.0)).show(ui, |ui| {
                    ui.label("Screen Overlay");
                    ui.add(egui::Checkbox::new(&mut self.screen, ""));
                    ui.end_row();

                    ui.label("Line Spacing");
                    ui.add(egui::Slider::new(&mut self.spacing, 0.0..=100.0));
                    ui.end_row();

                    ui.label("Thickness");
                    ui.add(egui::Slider::new(&mut self.thickness, 0.0..=5.0));
                    ui.end_row();

                    ui.label("Subdivisions");
                    ui.add(egui::Slider::new(&mut self.subdivisions, 5..=150));
                    ui.end_row();

                    ui.label("Min Opacity");
                    ui.add(egui::Slider::new(&mut self.min_opacity, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Max Opacity");
                    ui.add(egui::Slider::new(&mut self.max_opacity, 0.0..=1.0));
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
