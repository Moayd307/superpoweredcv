use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use superpoweredcv::generator::{self, ScrapedProfile};
use superpoweredcv::analysis::{ProfileConfig, InjectionPosition, Intensity, LowVisibilityPalette, OffpageOffset, InjectionContent};
use superpoweredcv::templates::GenerationType;
use superpoweredcv::config::AppConfig;
use superpoweredcv::llm::LlmClient;
use std::fs::File;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "SUPERGUI",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_styles(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
}

struct MyApp {
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    status_log: Vec<String>,
    
    // Injection
    injection_type: InjectionTypeGui,
    intensity: Intensity,
    position: InjectionPosition,
    
    // Content
    generation_type: GenerationType,
    phrases: Vec<String>,
    current_phrase: String,
    job_description: String,
    
    // Config
    config: AppConfig,
    show_settings: bool,
}

#[derive(PartialEq, Clone, Copy)]
enum InjectionTypeGui {
    None,
    VisibleMetaBlock,
    LowVisibilityBlock,
    OffpageLayer,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: None,
            status_log: vec!["> SYSTEM_READY".to_string()],
            injection_type: InjectionTypeGui::None,
            intensity: Intensity::Medium,
            position: InjectionPosition::Header,
            generation_type: GenerationType::Static,
            phrases: vec![],
            current_phrase: String::new(),
            job_description: String::new(),
            config: AppConfig::load(),
            show_settings: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::NONE.inner_margin(20.0)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("SUPERPOWERED_CV").size(32.0).strong().color(egui::Color32::from_rgb(255, 69, 0)));
                ui.add_space(5.0);
                ui.label(egui::RichText::new("TARGET: PDF_GENERATION_MODULE").monospace().color(egui::Color32::LIGHT_GRAY));
                ui.add_space(20.0);
            });

            // Settings Toggle
            if ui.button("SETTINGS").clicked() {
                self.show_settings = !self.show_settings;
            }

            if self.show_settings {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("LLM CONFIGURATION").strong());
                    ui.horizontal(|ui| {
                        ui.label("API URL:");
                        ui.text_edit_singleline(&mut self.config.llm.api_base_url);
                    });
                    ui.horizontal(|ui| {
                        ui.label("MODEL:");
                        ui.text_edit_singleline(&mut self.config.llm.model);
                    });
                    ui.horizontal(|ui| {
                        ui.label("API KEY:");
                        let mut key = self.config.llm.api_key.clone().unwrap_or_default();
                        ui.add(egui::TextEdit::singleline(&mut key).password(true));
                        self.config.llm.api_key = if key.is_empty() { None } else { Some(key) };
                    });
                    if ui.button("SAVE CONFIG").clicked() {
                        if let Err(e) = self.config.save() {
                            self.log(&format!("ERROR: CONFIG_SAVE_FAIL: {}", e));
                        } else {
                            self.log("CONFIG_SAVED");
                        }
                    }
                });
                ui.add_space(10.0);
            }

            // Input Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("INPUT_SOURCE:").strong().color(egui::Color32::WHITE));
                    if ui.button("SELECT_JSON").clicked() {
                        if let Some(path) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                            self.input_path = Some(path);
                            self.log("INPUT_SELECTED");
                        }
                    }
                    if let Some(path) = &self.input_path {
                        ui.label(egui::RichText::new(path.file_name().unwrap().to_string_lossy()).monospace().color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("NO_FILE").monospace().color(egui::Color32::DARK_GRAY));
                    }
                });
            });

            ui.add_space(10.0);

            // Output Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OUTPUT_DEST: ").strong().color(egui::Color32::WHITE));
                    if ui.button("SELECT_PATH").clicked() {
                        if let Some(path) = FileDialog::new().add_filter("pdf", &["pdf"]).save_file() {
                            self.output_path = Some(path);
                            self.log("OUTPUT_PATH_SET");
                        }
                    }
                    if let Some(path) = &self.output_path {
                        ui.label(egui::RichText::new(path.file_name().unwrap().to_string_lossy()).monospace().color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("NO_PATH").monospace().color(egui::Color32::DARK_GRAY));
                    }
                });
            });

            ui.add_space(10.0);

            // Injection Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("INJECTION_MODULE:").strong().color(egui::Color32::WHITE));
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("TYPE:");
                        egui::ComboBox::from_id_salt("injection_type")
                            .selected_text(match self.injection_type {
                                InjectionTypeGui::None => "NONE",
                                InjectionTypeGui::VisibleMetaBlock => "VISIBLE_META",
                                InjectionTypeGui::LowVisibilityBlock => "LOW_VISIBILITY",
                                InjectionTypeGui::OffpageLayer => "OFF_PAGE",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::None, "NONE");
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::VisibleMetaBlock, "VISIBLE_META");
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::LowVisibilityBlock, "LOW_VISIBILITY");
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::OffpageLayer, "OFF_PAGE");
                            });
                    });

                    if self.injection_type != InjectionTypeGui::None {
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("INTENSITY:");
                            egui::ComboBox::from_id_salt("intensity")
                                .selected_text(format!("{:?}", self.intensity).to_uppercase())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.intensity, Intensity::Soft, "SOFT");
                                    ui.selectable_value(&mut self.intensity, Intensity::Medium, "MEDIUM");
                                    ui.selectable_value(&mut self.intensity, Intensity::Aggressive, "AGGRESSIVE");
                                });
                        });

                        if self.injection_type == InjectionTypeGui::VisibleMetaBlock {
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("POSITION:");
                                let current_pos_text = match self.position {
                                    InjectionPosition::Header => "HEADER",
                                    InjectionPosition::Footer => "FOOTER",
                                    _ => "OTHER",
                                };
                                egui::ComboBox::from_id_salt("position")
                                    .selected_text(current_pos_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.position, InjectionPosition::Header, "HEADER");
                                        ui.selectable_value(&mut self.position, InjectionPosition::Footer, "FOOTER");
                                    });
                            });
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("CONTENT GENERATION:").strong());
                        
                        ui.horizontal(|ui| {
                            ui.label("MODE:");
                            egui::ComboBox::from_id_salt("gen_type")
                                .selected_text(format!("{:?}", self.generation_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.generation_type, GenerationType::Static, "Static");
                                    ui.selectable_value(&mut self.generation_type, GenerationType::LlmControl, "LLM Control");
                                    ui.selectable_value(&mut self.generation_type, GenerationType::Pollution, "Pollution");
                                    ui.selectable_value(&mut self.generation_type, GenerationType::AdTargeted, "Ad Targeted");
                                });
                        });

                        if self.generation_type == GenerationType::AdTargeted {
                            ui.label("JOB DESCRIPTION:");
                            ui.text_edit_multiline(&mut self.job_description);
                        }

                        if self.generation_type != GenerationType::Static {
                            if ui.button("GENERATE WITH LLM").clicked() {
                                self.generate_content();
                            }
                        }

                        ui.label("PHRASES:");
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.current_phrase);
                            if ui.button("ADD").clicked() && !self.current_phrase.is_empty() {
                                self.phrases.push(self.current_phrase.clone());
                                self.current_phrase.clear();
                            }
                        });

                        egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                            let mut to_remove = None;
                            for (i, phrase) in self.phrases.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("- {}", phrase));
                                    if ui.button("X").clicked() {
                                        to_remove = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = to_remove {
                                self.phrases.remove(i);
                            }
                        });
                    }
                });
            });

            ui.add_space(20.0);

            // Action Button
            let generate_btn = egui::Button::new(egui::RichText::new("INITIATE_GENERATION").size(18.0).strong())
                .min_size(egui::vec2(ui.available_width(), 50.0))
                .fill(egui::Color32::from_rgb(255, 69, 0)); // Fire Red

            if ui.add_enabled(self.input_path.is_some() && self.output_path.is_some(), generate_btn).clicked() {
                self.generate();
            }

            ui.add_space(20.0);

            // Console Log
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for log in &self.status_log {
                        ui.label(egui::RichText::new(log).monospace().size(12.0).color(egui::Color32::LIGHT_GRAY));
                    }
                });
            });
        });
    }
}

impl MyApp {
    fn log(&mut self, msg: &str) {
        self.status_log.push(format!("> {}", msg));
    }

    fn generate_content(&mut self) {
        self.log("CONTACTING LLM...");
        let client = LlmClient::new(self.config.llm.clone());
        
        let prompt = match self.generation_type {
            GenerationType::LlmControl => &self.config.prompts.control_sequence_generation,
            GenerationType::Pollution => &self.config.prompts.pollution_skills_generation,
            GenerationType::AdTargeted => &self.config.prompts.ad_targeted_pollution,
            _ => return,
        };

        let final_prompt = if self.generation_type == GenerationType::AdTargeted {
            prompt.replace("{job_description}", &self.job_description)
        } else {
            prompt.clone()
        };

        match client.generate(&final_prompt) {
            Ok(content) => {
                self.phrases.push(content);
                self.log("LLM: CONTENT_RECEIVED");
            }
            Err(e) => {
                self.log(&format!("ERROR: LLM_FAIL: {}", e));
            }
        }
    }

    fn generate(&mut self) {
        self.log("STARTING_SEQUENCE...");
        
        let input = self.input_path.as_ref().unwrap();
        let output = self.output_path.as_ref().unwrap();

        let file = match File::open(input) {
            Ok(f) => f,
            Err(e) => {
                self.log(&format!("ERROR: FAILED_TO_OPEN_INPUT: {}", e));
                return;
            }
        };

        let profile: ScrapedProfile = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(e) => {
                self.log(&format!("ERROR: JSON_PARSE_FAIL: {}", e));
                return;
            }
        };

        let content = InjectionContent {
            phrases: self.phrases.clone(),
            generation_type: self.generation_type.clone(),
            job_description: if self.generation_type == GenerationType::AdTargeted { Some(self.job_description.clone()) } else { None },
        };

        let injection_config = match self.injection_type {
            InjectionTypeGui::None => None,
            InjectionTypeGui::VisibleMetaBlock => Some(ProfileConfig::VisibleMetaBlock {
                position: self.position.clone(),
                intensity: self.intensity.clone(),
                content,
            }),
            InjectionTypeGui::LowVisibilityBlock => Some(ProfileConfig::LowVisibilityBlock {
                font_size_min: 1,
                font_size_max: 1,
                color_profile: LowVisibilityPalette::Gray,
                content,
            }),
            InjectionTypeGui::OffpageLayer => Some(ProfileConfig::OffpageLayer {
                offset_strategy: OffpageOffset::BottomClip,
                length: None,
                content,
            }),
        };

        match generator::generate_pdf(&profile, output, injection_config.as_ref()) {
            Ok(_) => self.log("SUCCESS: PDF_GENERATED"),
            Err(e) => self.log(&format!("ERROR: GENERATION_FAIL: {}", e)),
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Get the font names used for Monospace
    if let Some(monospace_fonts) = fonts.families.get(&egui::FontFamily::Monospace) {
        // Set Proportional to use the same fonts
        fonts.families.insert(egui::FontFamily::Proportional, monospace_fonts.clone());
    }

    ctx.set_fonts(fonts);
}

fn setup_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Brutalist Colors
    visuals.window_fill = egui::Color32::from_rgb(10, 10, 10); // Almost black
    visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    
    // Sharp edges
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;

    // High Contrast Borders
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0)); // Orange
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 69, 0)); // Red-Orange

    // Button Colors
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(40, 10, 10); // Dark Red tint
    
    // Selection
    visuals.selection.bg_fill = egui::Color32::from_rgb(255, 69, 0); // Red-Orange
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);

    ctx.set_visuals(visuals);
}
