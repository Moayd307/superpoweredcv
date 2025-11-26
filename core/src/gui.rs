use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use superpoweredcv::generator::{self, ScrapedProfile};
use superpoweredcv::analysis::{ProfileConfig, InjectionPosition, Intensity, LowVisibilityPalette, OffpageOffset, InjectionContent};
use superpoweredcv::templates::{GenerationType, default_templates};
use superpoweredcv::config::AppConfig;
use superpoweredcv::llm::LlmClient;
use superpoweredcv::pdf::{PdfMutator, RealPdfMutator, PdfMutationRequest};
use std::fs::File;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 800.0])
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

#[derive(PartialEq, Clone)]
enum InputSource {
    JsonFile(Option<PathBuf>),
    PdfFile(Option<PathBuf>),
    LinkedinUrl(String),
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum LlmProvider {
    OpenAI,
    Anthropic,
    Mistral,
    Groq,
    OpenRouter,
    LocalAI,
    Ollama,
    LMStudio,
    Custom,
}

#[derive(Clone)]
struct InjectionConfigGui {
    injection_type: InjectionTypeGui,
    intensity: Intensity,
    position: InjectionPosition,
    phrases: Vec<String>,
    current_phrase: String,
    generation_type: GenerationType,
    job_description: String,
}

impl Default for InjectionConfigGui {
    fn default() -> Self {
        Self {
            injection_type: InjectionTypeGui::VisibleMetaBlock,
            intensity: Intensity::Medium,
            position: InjectionPosition::Header,
            phrases: vec![],
            current_phrase: String::new(),
            generation_type: GenerationType::Static,
            job_description: String::new(),
        }
    }
}

struct MyApp {
    input_source: InputSource,
    output_path: Option<PathBuf>,
    status_log: Vec<String>,
    
    // Injections
    injections: Vec<InjectionConfigGui>,
    
    // Config
    config: AppConfig,
    show_settings: bool,
    selected_provider: LlmProvider,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum InjectionTypeGui {
    VisibleMetaBlock,
    LowVisibilityBlock,
    OffpageLayer,
    UnderlayText,
    StructuralFields,
    PaddingNoise,
    InlineJobAd,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            input_source: InputSource::JsonFile(None),
            output_path: None,
            status_log: vec!["> SYSTEM_READY".to_string()],
            injections: vec![],
            config: AppConfig::load(),
            show_settings: false,
            selected_provider: LlmProvider::LMStudio,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Settings Window
        if self.show_settings {
            egui::Window::new("CONFIGURATION_MATRIX")
                .open(&mut self.show_settings)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    self.render_settings(ui);
                });
        }

        egui::CentralPanel::default().frame(egui::Frame::NONE.inner_margin(20.0)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("SUPERPOWERED_CV").size(32.0).strong().color(egui::Color32::from_rgb(255, 69, 0)));
                ui.add_space(5.0);
                ui.label(egui::RichText::new("TARGET: PDF_GENERATION_MODULE").monospace().color(egui::Color32::LIGHT_GRAY));
                ui.add_space(20.0);
            });

            // Top Bar
            ui.horizontal(|ui| {
                if ui.button("⚙ SETTINGS").clicked() {
                    self.show_settings = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("v1.0.0-alpha").weak().small());
                });
            });
            ui.add_space(10.0);

            // Input Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("INPUT_SOURCE").strong().color(egui::Color32::WHITE));
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.input_source, InputSource::JsonFile(None), "JSON Profile");
                    ui.radio_value(&mut self.input_source, InputSource::PdfFile(None), "Existing PDF");
                    ui.radio_value(&mut self.input_source, InputSource::LinkedinUrl(String::new()), "LinkedIn URL");
                });

                ui.add_space(5.0);

                match &mut self.input_source {
                    InputSource::JsonFile(path) => {
                        ui.horizontal(|ui| {
                            if ui.button("SELECT JSON").clicked() {
                                if let Some(p) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                                    *path = Some(p);
                                    self.log("INPUT: JSON_SELECTED");
                                }
                            }
                            if let Some(p) = path {
                                ui.label(egui::RichText::new(p.file_name().unwrap().to_string_lossy()).color(egui::Color32::YELLOW));
                            } else {
                                ui.label("No file selected");
                            }
                        });
                    }
                    InputSource::PdfFile(path) => {
                        ui.horizontal(|ui| {
                            if ui.button("SELECT PDF").clicked() {
                                if let Some(p) = FileDialog::new().add_filter("pdf", &["pdf"]).pick_file() {
                                    *path = Some(p);
                                    self.log("INPUT: PDF_SELECTED");
                                }
                            }
                            if let Some(p) = path {
                                ui.label(egui::RichText::new(p.file_name().unwrap().to_string_lossy()).color(egui::Color32::YELLOW));
                            } else {
                                ui.label("No file selected");
                            }
                        });
                    }
                    InputSource::LinkedinUrl(url) => {
                        ui.horizontal(|ui| {
                            ui.label("URL:");
                            ui.text_edit_singleline(url);
                        });
                        ui.label(egui::RichText::new("Note: URL scraping requires external browser extension.").small().italics());
                    }
                }
            });

            ui.add_space(10.0);

            // Output Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OUTPUT_DEST: ").strong().color(egui::Color32::WHITE));
                    if ui.button("SELECT PATH").clicked() {
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

            // Injections List
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("INJECTION_MODULES").strong().color(egui::Color32::WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+ ADD MODULE").clicked() {
                            self.injections.push(InjectionConfigGui::default());
                        }
                    });
                });
                
                ui.separator();

                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    let mut to_remove = None;
                    for (idx, injection) in self.injections.iter_mut().enumerate() {
                        ui.push_id(idx, |ui| {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("MODULE #{}", idx + 1)).strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("REMOVE").clicked() {
                                            to_remove = Some(idx);
                                        }
                                    });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Type:");
                                    egui::ComboBox::from_id_salt("type")
                                        .selected_text(format!("{:?}", injection.injection_type))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::VisibleMetaBlock, "Visible Meta");
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::LowVisibilityBlock, "Low Visibility");
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::OffpageLayer, "Off-Page Layer");
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::UnderlayText, "Underlay Text");
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::StructuralFields, "Structural Fields");
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::PaddingNoise, "Padding Noise");
                                            ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::InlineJobAd, "Inline Job Ad");
                                        });
                                    
                                    ui.label("Intensity:");
                                    egui::ComboBox::from_id_salt("intensity")
                                        .selected_text(format!("{:?}", injection.intensity))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut injection.intensity, Intensity::Soft, "Soft");
                                            ui.selectable_value(&mut injection.intensity, Intensity::Medium, "Medium");
                                            ui.selectable_value(&mut injection.intensity, Intensity::Aggressive, "Aggressive");
                                            ui.selectable_value(&mut injection.intensity, Intensity::Custom, "Custom (No Template)");
                                        });
                                });

                                if injection.injection_type == InjectionTypeGui::VisibleMetaBlock {
                                    ui.horizontal(|ui| {
                                        ui.label("Position:");
                                        egui::ComboBox::from_id_salt("pos")
                                            .selected_text(format!("{:?}", injection.position))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut injection.position, InjectionPosition::Header, "Header");
                                                ui.selectable_value(&mut injection.position, InjectionPosition::Footer, "Footer");
                                            });
                                    });
                                }

                                ui.collapsing("Content Configuration", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Gen Mode:");
                                        egui::ComboBox::from_id_salt("gen")
                                            .selected_text(format!("{:?}", injection.generation_type))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut injection.generation_type, GenerationType::Static, "Static");
                                                ui.selectable_value(&mut injection.generation_type, GenerationType::LlmControl, "LLM Control");
                                                ui.selectable_value(&mut injection.generation_type, GenerationType::Pollution, "Pollution");
                                                ui.selectable_value(&mut injection.generation_type, GenerationType::AdTargeted, "Ad Targeted");
                                            });
                                    });

                                    if injection.generation_type == GenerationType::AdTargeted {
                                        ui.label("Job Description:");
                                        ui.text_edit_multiline(&mut injection.job_description);
                                    }

                                    if injection.generation_type != GenerationType::Static {
                                        if ui.button("GENERATE CONTENT (LLM)").clicked() {
                                            // Need to handle async or blocking call here. 
                                            // For now, we clone config and do it blocking (freezes UI briefly)
                                            let client = LlmClient::new(self.config.llm.clone());
                                            let prompt = match injection.generation_type {
                                                GenerationType::LlmControl => &self.config.prompts.control_sequence_generation,
                                                GenerationType::Pollution => &self.config.prompts.pollution_skills_generation,
                                                GenerationType::AdTargeted => &self.config.prompts.ad_targeted_pollution,
                                                _ => "",
                                            };
                                            let final_prompt = if injection.generation_type == GenerationType::AdTargeted {
                                                prompt.replace("{job_description}", &injection.job_description)
                                            } else {
                                                prompt.to_string()
                                            };
                                            
                                            match client.generate(&final_prompt) {
                                                Ok(c) => injection.phrases.push(c),
                                                Err(e) => self.log(&format!("LLM Error: {}", e)),
                                            }
                                        }
                                    }

                                    ui.label("Phrases:");
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut injection.current_phrase);
                                        if ui.button("Add").clicked() && !injection.current_phrase.is_empty() {
                                            injection.phrases.push(injection.current_phrase.clone());
                                            injection.current_phrase.clear();
                                        }
                                    });
                                    for (pi, p) in injection.phrases.iter().enumerate() {
                                        ui.label(format!("• {}", p));
                                    }
                                });
                            });
                        });
                        ui.add_space(5.0);
                    }
                    if let Some(i) = to_remove {
                        self.injections.remove(i);
                    }
                });
            });

            ui.add_space(20.0);

            // Action Button
            let generate_btn = egui::Button::new(egui::RichText::new("INITIATE GENERATION").size(18.0).strong())
                .min_size(egui::vec2(ui.available_width(), 50.0))
                .fill(egui::Color32::from_rgb(255, 69, 0));

            if ui.add_enabled(self.output_path.is_some(), generate_btn).clicked() {
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

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("LLM Provider Settings");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Provider:");
            egui::ComboBox::from_id_salt("provider")
                .selected_text(format!("{:?}", self.selected_provider))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::OpenAI, "OpenAI");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::Anthropic, "Anthropic");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::Mistral, "Mistral");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::Groq, "Groq");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::OpenRouter, "OpenRouter");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::LocalAI, "LocalAI");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::Ollama, "Ollama");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::LMStudio, "LM Studio");
                    ui.selectable_value(&mut self.selected_provider, LlmProvider::Custom, "Custom");
                });
        });

        // Auto-configure based on provider
        if ui.button("Apply Provider Defaults").clicked() {
            match self.selected_provider {
                LlmProvider::OpenAI => {
                    self.config.llm.api_base_url = "https://api.openai.com/v1".to_string();
                    self.config.llm.model = "gpt-4o".to_string();
                }
                LlmProvider::Anthropic => {
                    self.config.llm.api_base_url = "https://api.anthropic.com/v1".to_string();
                    self.config.llm.model = "claude-3-opus-20240229".to_string();
                }
                LlmProvider::Ollama => {
                    self.config.llm.api_base_url = "http://localhost:11434/v1".to_string();
                    self.config.llm.model = "llama3".to_string();
                }
                LlmProvider::LMStudio => {
                    self.config.llm.api_base_url = "http://localhost:1234/v1".to_string();
                    self.config.llm.model = "local-model".to_string();
                }
                _ => {}
            }
        }

        if matches!(self.selected_provider, LlmProvider::Ollama | LlmProvider::LMStudio | LlmProvider::LocalAI) {
            if ui.button("Auto-Detect Local Models").clicked() {
                // Simple check (simulated for now, could use reqwest)
                self.log("Checking localhost:11434 and localhost:1234...");
                // In a real app, we'd fire a request here.
                self.log("Auto-detection requires running service.");
            }
        }

        ui.separator();
        
        ui.label("API URL:");
        ui.text_edit_singleline(&mut self.config.llm.api_base_url);
        
        ui.label("Model Name:");
        ui.text_edit_singleline(&mut self.config.llm.model);
        
        ui.label("API Key:");
        let mut key = self.config.llm.api_key.clone().unwrap_or_default();
        ui.add(egui::TextEdit::singleline(&mut key).password(true));
        self.config.llm.api_key = if key.is_empty() { None } else { Some(key) };

        ui.separator();
        ui.heading("Prompt Templates");
        
        ui.label("Control Sequence Prompt:");
        ui.text_edit_multiline(&mut self.config.prompts.control_sequence_generation);
        
        ui.label("Pollution Skills Prompt:");
        ui.text_edit_multiline(&mut self.config.prompts.pollution_skills_generation);
        
        ui.label("Ad-Targeted Prompt:");
        ui.text_edit_multiline(&mut self.config.prompts.ad_targeted_pollution);

        ui.add_space(10.0);
        if ui.button("Save Configuration").clicked() {
            if let Err(e) = self.config.save() {
                self.log(&format!("Config Save Error: {}", e));
            } else {
                self.log("Configuration Saved.");
            }
        }
    }

    fn generate(&mut self) {
        self.log("STARTING PIPELINE...");
        
        // 1. Determine Base PDF
        let base_pdf_path = match &self.input_source {
            InputSource::JsonFile(Some(path)) => {
                // Generate temp PDF from JSON
                let file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => { self.log(&format!("Error opening JSON: {}", e)); return; }
                };
                let profile: ScrapedProfile = match serde_json::from_reader(file) {
                    Ok(p) => p,
                    Err(e) => { self.log(&format!("Error parsing JSON: {}", e)); return; }
                };
                
                let temp_path = std::env::temp_dir().join("superpoweredcv_temp.pdf");
                if let Err(e) = generator::generate_pdf(&profile, &temp_path, None) {
                    self.log(&format!("Error generating base PDF: {}", e));
                    return;
                }
                temp_path
            }
            InputSource::PdfFile(Some(path)) => path.clone(),
            InputSource::LinkedinUrl(_) => {
                self.log("Error: URL input not implemented yet.");
                return;
            }
            _ => {
                self.log("Error: No input selected.");
                return;
            }
        };

        // 2. Build Profiles
        let mut profiles = Vec::new();
        for inj in &self.injections {
            let content = InjectionContent {
                phrases: inj.phrases.clone(),
                generation_type: inj.generation_type.clone(),
                job_description: if inj.generation_type == GenerationType::AdTargeted { Some(inj.job_description.clone()) } else { None },
            };

            let profile = match inj.injection_type {
                InjectionTypeGui::VisibleMetaBlock => ProfileConfig::VisibleMetaBlock {
                    position: inj.position.clone(),
                    intensity: inj.intensity.clone(),
                    content,
                },
                InjectionTypeGui::LowVisibilityBlock => ProfileConfig::LowVisibilityBlock {
                    font_size_min: 1,
                    font_size_max: 1,
                    color_profile: LowVisibilityPalette::Gray,
                    content,
                },
                InjectionTypeGui::OffpageLayer => ProfileConfig::OffpageLayer {
                    offset_strategy: OffpageOffset::BottomClip,
                    length: None,
                    content,
                },
                InjectionTypeGui::UnderlayText => ProfileConfig::UnderlayText,
                InjectionTypeGui::StructuralFields => ProfileConfig::StructuralFields {
                    targets: vec![superpoweredcv::analysis::StructuralTarget::PdfTag], // Default for now
                },
                InjectionTypeGui::PaddingNoise => ProfileConfig::PaddingNoise {
                    padding_tokens_before: Some(100),
                    padding_tokens_after: Some(100),
                    padding_style: superpoweredcv::analysis::PaddingStyle::JobRelated,
                },
                InjectionTypeGui::InlineJobAd => ProfileConfig::InlineJobAd {
                    job_ad_source: superpoweredcv::analysis::JobAdSource::Inline,
                    placement: superpoweredcv::analysis::JobAdPlacement::Back,
                    ad_excerpt_ratio: 1.0,
                },
            };
            profiles.push(profile);
        }

        // 3. Mutate
        let output = self.output_path.as_ref().unwrap();
        let mutator = RealPdfMutator::new(output.parent().unwrap());
        
        let request = PdfMutationRequest {
            base_pdf: base_pdf_path,
            profiles,
            template: default_templates().get("default").unwrap().clone(), // Fallback template
            variant_id: Some(output.file_stem().unwrap().to_string_lossy().to_string()),
        };

        match mutator.mutate(request) {
            Ok(res) => {
                // Move result to final output if needed (mutator saves to output_dir/variant_id.pdf)
                // We want to save to `output` path exactly.
                if let Err(e) = std::fs::rename(&res.mutated_pdf, output) {
                    self.log(&format!("Error moving file: {}", e));
                } else {
                    self.log("SUCCESS: PDF Generated & Injected.");
                }
            }
            Err(e) => self.log(&format!("Error mutating PDF: {}", e)),
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(monospace_fonts) = fonts.families.get(&egui::FontFamily::Monospace) {
        fonts.families.insert(egui::FontFamily::Proportional, monospace_fonts.clone());
    }
    ctx.set_fonts(fonts);
}

fn setup_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(10, 10, 10);
    visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0));
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 69, 0));
    visuals.selection.bg_fill = egui::Color32::from_rgb(255, 69, 0);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    ctx.set_visuals(visuals);
}
