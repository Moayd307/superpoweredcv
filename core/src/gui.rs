use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use superpoweredcv::generator::{self, ScrapedProfile};
use superpoweredcv::analysis::{ProfileConfig, InjectionPosition, Intensity, LowVisibilityPalette, OffpageOffset, InjectionContent};
use superpoweredcv::templates::{GenerationType, default_templates};
use superpoweredcv::config::AppConfig;
use superpoweredcv::llm::LlmClient;
use superpoweredcv::pdf::{PdfMutator, RealPdfMutator, PdfMutationRequest};
use superpoweredcv::latex::LatexResume;
use std::fs::File;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 800.0])
            .with_resizable(true)
            .with_decorations(false) // Custom window frame
            .with_transparent(true),
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
    Gemini,
    Cohere,
    DeepSeek,
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

    // Latex Builder
    show_latex_builder: bool,
    latex_resume: LatexResume,

    // Log Window
    show_log_window: bool,
    
    // Window States
    settings_pinned: bool,
    builder_pinned: bool,
    logs_pinned: bool,
    main_pinned: bool,
    preview_pinned: bool,
    
    // Preview
    show_injection_preview: bool,
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
    TrackingPixel,
    CodeInjection,
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
            show_latex_builder: false,
            latex_resume: LatexResume::default(),
            show_log_window: false,
            
            settings_pinned: false,
            builder_pinned: false,
            logs_pinned: false,
            main_pinned: false,
            preview_pinned: false,
            
            show_injection_preview: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main Window Custom Frame
        let mut pinned = self.main_pinned;
        custom_window_frame(ctx, "SUPERPOWERED_CV", |ui| {
            self.render_main_content(ui);
        }, &mut pinned);
        self.main_pinned = pinned;

        // Settings Window
        if self.show_settings {
            let mut pinned = self.settings_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("CONFIGURATION_MATRIX")
                .with_inner_size([500.0, 600.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("settings_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "CONFIGURATION_MATRIX", |ui| {
                        self.render_settings(ui);
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_settings = false;
                    }
                }
            );
            self.settings_pinned = pinned;
        }

        // Latex Builder Window
        if self.show_latex_builder {
            let mut pinned = self.builder_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("LATEX_VISUAL_BUILDER")
                .with_inner_size([1000.0, 800.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("latex_builder_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "LATEX_VISUAL_BUILDER", |ui| {
                        self.render_latex_builder(ui);
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_latex_builder = false;
                    }
                }
            );
            self.builder_pinned = pinned;
        }

        // Log Window
        if self.show_log_window {
            let mut pinned = self.logs_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("SYSTEM_LOGS")
                .with_inner_size([400.0, 500.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("log_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "SYSTEM_LOGS", |ui| {
                        ui.heading(egui::RichText::new("/// SYSTEM EVENT LOG ///").strong().color(egui::Color32::from_rgb(0, 255, 0)));
                        ui.separator();
                        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            for log in &self.status_log {
                                ui.label(egui::RichText::new(log).monospace().size(10.0));
                            }
                        });
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_log_window = false;
                    }
                }
            );
            self.logs_pinned = pinned;
        }

        // Preview Window
        if self.show_injection_preview {
            let mut pinned = self.preview_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("INJECTION_PREVIEW")
                .with_inner_size([600.0, 800.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("preview_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "INJECTION_PREVIEW", |ui| {
                        self.render_preview(ui);
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_injection_preview = false;
                    }
                }
            );
            self.preview_pinned = pinned;
        }
    }
}

impl MyApp {
    fn render_preview(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("VISUAL_DIAGNOSTIC_MODE").strong());
        let (rect, _resp) = ui.allocate_at_least(ui.available_size(), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        
        // Draw Page Background
        painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::BLACK), egui::StrokeKind::Inside);
        
        // Draw Dummy Text Lines
        for i in 0..30 {
            let y = rect.min.y + 40.0 + (i as f32 * 15.0);
            if y < rect.max.y - 40.0 {
                painter.line_segment(
                    [egui::pos2(rect.min.x + 30.0, y), egui::pos2(rect.max.x - 30.0, y)],
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY)
                );
            }
        }

        // Draw Injections
        for (idx, injection) in self.injections.iter().enumerate() {
            let color = match idx % 3 {
                0 => egui::Color32::from_rgba_premultiplied(255, 0, 0, 150),
                1 => egui::Color32::from_rgba_premultiplied(0, 255, 0, 150),
                _ => egui::Color32::from_rgba_premultiplied(0, 0, 255, 150),
            };
            
            match injection.injection_type {
                InjectionTypeGui::VisibleMetaBlock => {
                    let y = match injection.position {
                        InjectionPosition::Header => rect.min.y + 10.0,
                        InjectionPosition::Footer => rect.max.y - 30.0,
                        _ => rect.min.y + 100.0,
                    };
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(rect.min.x + 10.0, y), egui::vec2(rect.width() - 20.0, 20.0)),
                        0.0,
                        color
                    );
                    painter.text(egui::pos2(rect.min.x + 15.0, y + 10.0), egui::Align2::LEFT_CENTER, format!("Module #{}", idx+1), egui::FontId::default(), egui::Color32::BLACK);
                }
                InjectionTypeGui::LowVisibilityBlock => {
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(rect.min.x + 10.0, rect.max.y - 10.0), egui::vec2(rect.width() - 20.0, 5.0)),
                        0.0,
                        color
                    );
                }
                InjectionTypeGui::TrackingPixel => {
                    painter.circle_filled(egui::pos2(rect.max.x - 20.0, rect.max.y - 20.0), 5.0, color);
                    painter.text(egui::pos2(rect.max.x - 30.0, rect.max.y - 20.0), egui::Align2::RIGHT_CENTER, "TRACKER", egui::FontId::default(), egui::Color32::BLACK);
                }
                InjectionTypeGui::CodeInjection => {
                    painter.rect_stroke(
                        egui::Rect::from_min_size(egui::pos2(rect.min.x + 50.0, rect.min.y + 200.0), egui::vec2(rect.width() - 100.0, 50.0)),
                        0.0,
                        egui::Stroke::new(2.0, color),
                        egui::StrokeKind::Inside
                    );
                    painter.text(egui::pos2(rect.center().x, rect.min.y + 225.0), egui::Align2::CENTER_CENTER, "MALICIOUS_PAYLOAD", egui::FontId::default(), egui::Color32::BLACK);
                }
                _ => {}
            }
        }
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui) {
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
            if ui.button("📝 LATEX BUILDER").clicked() {
                self.show_latex_builder = true;
            }
            if ui.button("📋 LOGS").clicked() {
                self.show_log_window = true;
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

            let mut log_msg = None;
            match &mut self.input_source {
                InputSource::JsonFile(path) => {
                    ui.horizontal(|ui| {
                        if ui.button("SELECT JSON").clicked() {
                            if let Some(p) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                                *path = Some(p);
                                log_msg = Some("INPUT: JSON_SELECTED");
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
                                log_msg = Some("INPUT: PDF_SELECTED");
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
            if let Some(msg) = log_msg {
                self.log(msg);
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

        // Injection Modules
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("INJECTION_MODULES").strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✚ ADD MODULE").clicked() {
                        self.injections.push(InjectionConfigGui::default());
                    }
                    if ui.button("👁 PREVIEW").clicked() {
                        self.show_injection_preview = !self.show_injection_preview;
                    }
                });
            });
            
            ui.separator();

            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                let mut to_remove = None;
                let mut pending_error = None;
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
                                        ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::TrackingPixel, "Tracking Pixel");
                                        ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::CodeInjection, "Code Injection");
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
                                            Err(e) => pending_error = Some(format!("LLM Error: {}", e)),
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
                                for (_pi, p) in injection.phrases.iter().enumerate() {
                                    ui.label(format!("• {}", p));
                                }
                            });
                        });
                    });
                }
                
                if let Some(idx) = to_remove {
                    self.injections.remove(idx);
                }
                if let Some(e) = pending_error {
                    self.log(&e);
                }
            });
        });

        ui.add_space(20.0);

        // Action Button
        ui.vertical_centered(|ui| {
            let btn = egui::Button::new(egui::RichText::new("⚡ INJECT & GENERATE ⚡").size(20.0).strong().color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(255, 69, 0))
                .min_size(egui::vec2(200.0, 50.0));
            
            if ui.add(btn).clicked() {
                self.generate();
            }
        });

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
                    let mut changed = false;
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::OpenAI, "OpenAI").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Anthropic, "Anthropic").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Mistral, "Mistral").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Groq, "Groq").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::OpenRouter, "OpenRouter").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::LocalAI, "LocalAI").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Ollama, "Ollama").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::LMStudio, "LM Studio").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Gemini, "Gemini").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Cohere, "Cohere").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::DeepSeek, "DeepSeek").clicked();
                    changed |= ui.selectable_value(&mut self.selected_provider, LlmProvider::Custom, "Custom").clicked();

                    if changed {
                        match self.selected_provider {
                            LlmProvider::OpenAI => {
                                self.config.llm.api_base_url = "https://api.openai.com/v1".to_string();
                                self.config.llm.model = "gpt-4o".to_string();
                            }
                            LlmProvider::Anthropic => {
                                self.config.llm.api_base_url = "https://api.anthropic.com/v1".to_string();
                                self.config.llm.model = "claude-3-opus-20240229".to_string();
                            }
                            LlmProvider::Mistral => {
                                self.config.llm.api_base_url = "https://api.mistral.ai/v1".to_string();
                                self.config.llm.model = "mistral-large-latest".to_string();
                            }
                            LlmProvider::Groq => {
                                self.config.llm.api_base_url = "https://api.groq.com/openai/v1".to_string();
                                self.config.llm.model = "llama3-70b-8192".to_string();
                            }
                            LlmProvider::OpenRouter => {
                                self.config.llm.api_base_url = "https://openrouter.ai/api/v1".to_string();
                                self.config.llm.model = "openai/gpt-4o".to_string();
                            }
                            LlmProvider::LocalAI => {
                                self.config.llm.api_base_url = "http://localhost:8080/v1".to_string();
                                self.config.llm.model = "gpt-4".to_string();
                            }
                            LlmProvider::Ollama => {
                                self.config.llm.api_base_url = "http://localhost:11434/v1".to_string();
                                self.config.llm.model = "llama3".to_string();
                            }
                            LlmProvider::LMStudio => {
                                self.config.llm.api_base_url = "http://localhost:1234/v1".to_string();
                                self.config.llm.model = "local-model".to_string();
                            }
                            LlmProvider::Gemini => {
                                self.config.llm.api_base_url = "https://generativelanguage.googleapis.com/v1beta/openai".to_string();
                                self.config.llm.model = "gemini-1.5-pro".to_string();
                            }
                            LlmProvider::Cohere => {
                                self.config.llm.api_base_url = "https://api.cohere.com/v1".to_string();
                                self.config.llm.model = "command-r-plus".to_string();
                            }
                            LlmProvider::DeepSeek => {
                                self.config.llm.api_base_url = "https://api.deepseek.com/v1".to_string();
                                self.config.llm.model = "deepseek-chat".to_string();
                            }
                            _ => {}
                        }
                    }
                });
        });

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
                    content,
                },
                InjectionTypeGui::UnderlayText => ProfileConfig::UnderlayText,
                InjectionTypeGui::StructuralFields => ProfileConfig::StructuralFields {
                    targets: vec![superpoweredcv::analysis::StructuralTarget::PdfTag], // Default for now
                },
                InjectionTypeGui::PaddingNoise => ProfileConfig::PaddingNoise {
                    padding_tokens_before: 100,
                    padding_tokens_after: 100,
                    padding_style: superpoweredcv::analysis::PaddingStyle::JobRelated,
                },
                InjectionTypeGui::InlineJobAd => ProfileConfig::InlineJobAd {
                    job_ad_source: superpoweredcv::analysis::JobAdSource::Inline,
                    placement: superpoweredcv::analysis::JobAdPlacement::Back,
                    ad_excerpt_ratio: 1.0,
                },
                InjectionTypeGui::TrackingPixel => ProfileConfig::TrackingPixel {
                    url: "https://canarytokens.org/pixel".to_string(), // Default placeholder
                },
                InjectionTypeGui::CodeInjection => ProfileConfig::CodeInjection {
                    payload: "alert('XSS')".to_string(), // Default placeholder
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
            template: default_templates().into_iter().find(|t| t.id == "default").unwrap_or_else(|| default_templates()[0].clone()), // Fallback template
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

    fn render_latex_builder(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            // Left Column: Editor
            columns[0].vertical(|ui| {
                ui.heading("Content Editor");
                
                ui.horizontal(|ui| {
                    if ui.button("📥 Import from Input").clicked() {
                        // Try to load from input source
                        if let InputSource::JsonFile(Some(path)) = &self.input_source {
                             if let Ok(file) = File::open(path) {
                                 if let Ok(profile) = serde_json::from_reader::<_, ScrapedProfile>(file) {
                                     self.latex_resume.import_from_profile(&profile);
                                 }
                             }
                        }
                    }
                    
                    ui.label("Font:");
                    egui::ComboBox::from_id_salt("font")
                        .selected_text(if self.latex_resume.font.is_empty() { "Default" } else { &self.latex_resume.font })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.latex_resume.font, "".to_string(), "Default");
                            ui.selectable_value(&mut self.latex_resume.font, "helvet".to_string(), "Helvetica");
                            ui.selectable_value(&mut self.latex_resume.font, "mathpazo".to_string(), "Palatino");
                            ui.selectable_value(&mut self.latex_resume.font, "avant".to_string(), "Avant Garde");
                            ui.selectable_value(&mut self.latex_resume.font, "bookman".to_string(), "Bookman");
                            ui.selectable_value(&mut self.latex_resume.font, "charter".to_string(), "Charter");
                        });
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.collapsing("Personal Info", |ui| {
                        ui.text_edit_singleline(&mut self.latex_resume.personal_info.name).on_hover_text("Name");
                        ui.text_edit_singleline(&mut self.latex_resume.personal_info.email).on_hover_text("Email");
                        ui.text_edit_singleline(&mut self.latex_resume.personal_info.phone).on_hover_text("Phone");
                        ui.text_edit_singleline(&mut self.latex_resume.personal_info.linkedin).on_hover_text("LinkedIn");
                        ui.text_edit_singleline(&mut self.latex_resume.personal_info.github).on_hover_text("GitHub");
                    });

                    ui.separator();
                    ui.heading("Sections");
                    ui.label(egui::RichText::new("Drag sections to reorder (Not implemented in this version)").small().italics());
                    
                    let mut section_to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;

                    for (idx, section) in self.latex_resume.sections.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{}", idx+1));
                                if ui.button("⬆").clicked() { move_up = Some(idx); }
                                if ui.button("⬇").clicked() { move_down = Some(idx); }
                                ui.text_edit_singleline(&mut section.title);
                                if ui.button("X").clicked() {
                                    section_to_remove = Some(idx);
                                }
                            });
                            
                            let mut item_to_remove = None;
                            for (i_idx, item) in section.items.iter_mut().enumerate() {
                                ui.separator();
                                ui.text_edit_singleline(&mut item.title).on_hover_text("Title");
                                ui.text_edit_singleline(&mut item.subtitle).on_hover_text("Subtitle");
                                ui.text_edit_singleline(&mut item.date).on_hover_text("Date");
                                
                                ui.label("Description Points:");
                                let mut desc_to_remove = None;
                                for (d_idx, desc) in item.description.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(desc);
                                        if ui.button("-").clicked() {
                                            desc_to_remove = Some(d_idx);
                                        }
                                    });
                                }
                                if let Some(d) = desc_to_remove {
                                    item.description.remove(d);
                                }
                                if ui.button("+ Add Point").clicked() {
                                    item.description.push(String::new());
                                }

                                if ui.button("Remove Item").clicked() {
                                    item_to_remove = Some(i_idx);
                                }
                            }
                            if let Some(i) = item_to_remove {
                                section.items.remove(i);
                            }
                            if ui.button("+ Add Item").clicked() {
                                section.items.push(superpoweredcv::latex::SectionItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    title: "New Item".to_string(),
                                    subtitle: "Subtitle".to_string(),
                                    date: "Date".to_string(),
                                    description: vec![],
                                });
                            }
                        });
                    }
                    if let Some(s) = section_to_remove {
                        self.latex_resume.sections.remove(s);
                    }
                    if let Some(idx) = move_up {
                        if idx > 0 {
                            self.latex_resume.sections.swap(idx, idx - 1);
                        }
                    }
                    if let Some(idx) = move_down {
                        if idx < self.latex_resume.sections.len() - 1 {
                            self.latex_resume.sections.swap(idx, idx + 1);
                        }
                    }
                    
                    if ui.button("+ Add Section").clicked() {
                        self.latex_resume.sections.push(superpoweredcv::latex::ResumeSection {
                            id: uuid::Uuid::new_v4().to_string(),
                            title: "New Section".to_string(),
                            items: vec![],
                        });
                    }
                });
            });

            // Right Column: Preview
            columns[1].vertical(|ui| {
                ui.heading("LaTeX Preview");
                let latex_code = self.latex_resume.generate_latex();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut latex_code.as_str()).code_editor().desired_width(f32::INFINITY));
                });
                
                ui.horizontal(|ui| {
                    if ui.button("COPY TO CLIPBOARD").clicked() {
                        ui.ctx().copy_text(latex_code.clone());
                    }
                    if ui.button("EXPORT PDF").clicked() {
                        // Save to temp file and run pdflatex
                        if let Some(path) = FileDialog::new().set_file_name("resume.pdf").save_file() {
                            let tex_path = path.with_extension("tex");
                            if std::fs::write(&tex_path, &latex_code).is_ok() {
                                // Try to run pdflatex
                                match std::process::Command::new("pdflatex")
                                    .arg("-output-directory")
                                    .arg(path.parent().unwrap())
                                    .arg(&tex_path)
                                    .output() {
                                        Ok(output) => {
                                            if output.status.success() {
                                                // self.log("PDF Export Successful"); // Can't log easily here without refactor or passing log queue
                                            } else {
                                                // self.log("PDF Export Failed (pdflatex error)");
                                            }
                                        },
                                        Err(_) => {
                                            // self.log("PDF Export Failed (pdflatex not found?)");
                                        }
                                    }
                            }
                        }
                    }
                });
            });
        });
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
    // Brutalist Palette
    let bg_color = egui::Color32::from_rgb(15, 15, 15);
    let fg_color = egui::Color32::from_rgb(240, 240, 240);
    let accent_color = egui::Color32::from_rgb(255, 50, 50); // Red
    let border_color = egui::Color32::from_rgb(80, 80, 80);

    visuals.window_fill = bg_color;
    visuals.panel_fill = bg_color;
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.window_stroke = egui::Stroke::new(2.0, border_color);
    
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border_color);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg_color);
    
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_color);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg_color);
    
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, accent_color);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg_color);
    
    visuals.widgets.active.bg_fill = accent_color;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, fg_color);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    
    visuals.selection.bg_fill = accent_color;
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    
    ctx.set_visuals(visuals);
    
    // Spacing
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(15);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    ctx.set_style(style);
}

fn custom_window_frame(
    ctx: &egui::Context,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
    pinned: &mut bool,
) {
    use egui::*;
    let panel_frame = Frame {
        fill: ctx.style().visuals.window_fill(),
        corner_radius: 10.into(),
        stroke: ctx.style().visuals.window_stroke(),
        ..Default::default()
    };

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, title_bar_rect, title, pinned);

        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        };
        
        let mut content_ui = ui.child_ui(content_rect, *ui.layout(), None);
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    title_bar_rect: egui::Rect,
    title: &str,
    pinned: &mut bool,
) {
    use egui::*;

    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click_and_drag());

    painter.rect_filled(
        title_bar_rect,
        CornerRadius {
            nw: 10,
            ne: 10,
            sw: 0,
            se: 0,
        },
        ui.visuals().widgets.inactive.bg_fill,
    );

    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    if title_bar_response.double_clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
    } else if title_bar_response.is_pointer_button_down_on() {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);

            if ui.add(Button::new("❌").frame(false)).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            
            let (maximize_text, maximize_cmd) = if ui.input(|i| i.viewport().maximized.unwrap_or(false)) {
                ("🗗", ViewportCommand::Maximized(false))
            } else {
                ("🗖", ViewportCommand::Maximized(true))
            };

            if ui.add(Button::new(maximize_text).frame(false)).clicked() {
                ui.ctx().send_viewport_cmd(maximize_cmd);
            }

            if ui.add(Button::new("🗕").frame(false)).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            
            let pin_text = if *pinned { "📌" } else { "📍" };
            if ui.add(Button::new(pin_text).frame(false)).clicked() {
                *pinned = !*pinned;
            }
        });
    });
}
