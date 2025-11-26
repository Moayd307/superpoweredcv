use clap::{Parser, Subcommand};
use config::{Config, File};
use std::path::PathBuf;
mod gui;
use superpoweredcv::pipeline::{LoggingConfig, LogField, MetricSpec, MetricType, PipelineConfig, PipelineType};
use superpoweredcv::analysis::{
    AnalysisPlan, AnalysisScenario, Intensity, InjectionPosition, JobAdPlacement, JobAdSource,
    PaddingStyle, ProfileConfig, AnalysisEngine,
};
use superpoweredcv::templates::default_templates;
use superpoweredcv::generator::{self, ScrapedProfile};
use lopdf::dictionary;
use std::fs::File as StdFile;

#[derive(Parser)]
#[command(name = "superpoweredcv")]
#[command(about = "SuperpoweredCV CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to configuration file (yaml, json, toml)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an analysis scenario
    Analyze {
        /// Path to the scenario definition file
        #[arg(short, long)]
        scenario: Option<PathBuf>,
    },
    /// Run the built-in demo scenario
    Demo,
    /// Validate a configuration file
    Validate,
    /// Generate a PDF from a scraped profile JSON
    Generate {
        /// Path to the profile JSON file
        #[arg(short, long)]
        profile: PathBuf,
        /// Output PDF path
        #[arg(short, long)]
        output: PathBuf,

        /// Injection type
        #[arg(long, value_enum, default_value_t = CliInjectionType::None)]
        injection: CliInjectionType,

        /// Intensity
        #[arg(long, value_enum, default_value_t = CliIntensity::Medium)]
        intensity: CliIntensity,

        /// Position (for VisibleMeta)
        #[arg(long, value_enum, default_value_t = CliPosition::Header)]
        position: CliPosition,

        /// Phrases to inject (for Static generation)
        #[arg(long)]
        phrases: Vec<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliInjectionType {
    None,
    VisibleMeta,
    LowVis,
    Offpage,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliIntensity {
    Soft,
    Medium,
    Aggressive,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliPosition {
    Header,
    Footer,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Analyze { scenario }) => {
            if let Some(path) = scenario {
                run_scenario_from_file(path);
            } else {
                eprintln!("Error: --scenario argument is required for 'analyze' command.");
            }
        }
        Some(Commands::Demo) => {
            run_demo_scenario();
        }
        Some(Commands::Validate) => {
            if let Some(config_path) = &cli.config {
                validate_config(config_path);
            } else {
                eprintln!("Error: --config argument is required for 'validate' command.");
            }
        }
        Some(Commands::Generate { profile, output, injection, intensity, position, phrases }) => {
            generate_pdf_from_json(profile, output, injection, intensity, position, phrases);
        }
        None => {
            println!("Starting GUI...");
            if let Err(e) = gui::run_gui() {
                eprintln!("GUI Error: {}", e);
            }
        }
    }
}

use superpoweredcv::pdf::{PdfMutator, RealPdfMutator, PdfMutationRequest};

fn generate_pdf_from_json(
    profile_path: &PathBuf, 
    output_path: &PathBuf,
    injection: &CliInjectionType,
    intensity: &CliIntensity,
    position: &CliPosition,
    phrases: &Vec<String>
) {
    let file = match StdFile::open(profile_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open profile file: {}", e);
            return;
        }
    };
    
    let profile: ScrapedProfile = match serde_json::from_reader(file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse profile JSON: {}", e);
            return;
        }
    };

    // 1. Generate Clean PDF
    let temp_path = std::env::temp_dir().join("superpoweredcv_cli_temp.pdf");
    if let Err(e) = generator::generate_pdf(&profile, &temp_path, None) {
        eprintln!("Failed to generate base PDF: {}", e);
        return;
    }

    // 2. Prepare Injection
    let content = superpoweredcv::analysis::InjectionContent {
        phrases: phrases.clone(),
        generation_type: superpoweredcv::templates::GenerationType::Static,
        job_description: None,
    };

    let injection_config = match injection {
        CliInjectionType::None => None,
        CliInjectionType::VisibleMeta => Some(ProfileConfig::VisibleMetaBlock {
            position: match position {
                CliPosition::Header => InjectionPosition::Header,
                CliPosition::Footer => InjectionPosition::Footer,
            },
            intensity: match intensity {
                CliIntensity::Soft => Intensity::Soft,
                CliIntensity::Medium => Intensity::Medium,
                CliIntensity::Aggressive => Intensity::Aggressive,
            },
            content,
        }),
        CliInjectionType::LowVis => Some(ProfileConfig::LowVisibilityBlock {
            font_size_min: 1,
            font_size_max: 1,
            color_profile: superpoweredcv::analysis::LowVisibilityPalette::Gray,
            content,
        }),
        CliInjectionType::Offpage => Some(ProfileConfig::OffpageLayer {
            offset_strategy: superpoweredcv::analysis::OffpageOffset::BottomClip,
            length: None,
            content,
        }),
    };

    if let Some(config) = injection_config {
        let mutator = RealPdfMutator::new(output_path.parent().unwrap());
        let request = PdfMutationRequest {
            base_pdf: temp_path,
            profiles: vec![config],
            template: default_templates().get("default").unwrap().clone(),
            variant_id: Some(output_path.file_stem().unwrap().to_string_lossy().to_string()),
        };

        match mutator.mutate(request) {
            Ok(res) => {
                // Rename result to final output
                if let Err(e) = std::fs::rename(&res.mutated_pdf, output_path) {
                    eprintln!("Failed to move output file: {}", e);
                } else {
                    println!("PDF generated and injected successfully at {}", output_path.display());
                }
            }
            Err(e) => eprintln!("Failed to inject PDF: {}", e),
        }
    } else {
        // Just move the temp file if no injection
        if let Err(e) = std::fs::rename(&temp_path, output_path) {
            eprintln!("Failed to move output file: {}", e);
        } else {
            println!("Clean PDF generated successfully at {}", output_path.display());
        }
    }
}

fn run_scenario_from_file(path: &PathBuf) {
    println!("Loading scenario from: {}", path.display());
    
    let settings = Config::builder()
        .add_source(File::from(path.clone()))
        .build();

    match settings {
        Ok(config) => {
            match config.try_deserialize::<AnalysisScenario>() {
                Ok(scenario) => {
                    let engine = AnalysisEngine::new(default_templates());
                    println!("Starting Analysis Scenario: {}", scenario.scenario_id);
                    match engine.run_scenario(&scenario) {
                        Ok(report) => print_report(&report),
                        Err(e) => eprintln!("Analysis failed: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to parse scenario: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to load config file: {}", e),
    }
}

fn validate_config(path: &PathBuf) {
    println!("Validating config: {}", path.display());
    let settings = Config::builder()
        .add_source(File::from(path.clone()))
        .build();

    match settings {
        Ok(_) => println!("Config file is valid."),
        Err(e) => eprintln!("Config file is invalid: {}", e),
    }
}

fn run_demo_scenario() {
    // Define a sample scenario
    let base_pdf_path = PathBuf::from("examples/clean_resume.pdf");
    ensure_demo_pdf(&base_pdf_path);

    let scenario = AnalysisScenario {
        scenario_id: "ats_pdf_analysis_smoke".into(),
        base_pdf: base_pdf_path,
        plans: vec![
            // Plan 1: Soft bias in the footer
            AnalysisPlan {
                profile: ProfileConfig::VisibleMetaBlock {
                    position: InjectionPosition::Footer,
                    intensity: Intensity::Soft,
                    content: Default::default(),
                },
                template_id: "soft_bias".into(),
            },
            // Plan 2: Aggressive override with padding noise
            AnalysisPlan {
                profile: ProfileConfig::PaddingNoise {
                    padding_tokens_before: Some(256),
                    padding_tokens_after: Some(256),
                    padding_style: PaddingStyle::JobRelated,
                },
                template_id: "aggressive_override".into(),
            },
            // Plan 3: Inline job ad injection
            AnalysisPlan {
                profile: ProfileConfig::InlineJobAd {
                    job_ad_source: JobAdSource::Inline,
                    placement: JobAdPlacement::AfterSummary,
                    ad_excerpt_ratio: 0.5,
                },
                template_id: "override_conflict".into(),
            },
        ],
        // Configure the target pipeline (simulated)
        pipeline: PipelineConfig {
            pipeline_type: PipelineType::HttpLlm {
                endpoint: "https://example-ats-llm/api/score".into(),
                prompt_template: Some("prompts/ats_prompt.txt".into()),
            },
            target: Some("candidate_scoring_service_v2".into()),
        },
        // Define metrics to track
        metrics: vec![
            MetricSpec {
                name: "score_shift".into(),
                metric_type: MetricType::NumericDiff,
                baseline: Some(0.0),
            },
        ],
        // Configure logging
        logging: Some(LoggingConfig {
            capture: vec![LogField::PdfVariantHash, LogField::RawLlmResponse],
        }),
    };

    // Initialize the engine with default templates
    let engine = AnalysisEngine::new(default_templates());

    println!("Starting Demo Analysis Scenario: {}", scenario.scenario_id);

    // Run the scenario
    match engine.run_scenario(&scenario) {
        Ok(report) => print_report(&report),
        Err(e) => eprintln!("Scenario failed: {}", e),
    }
}

fn print_report(report: &superpoweredcv::analysis::ScenarioReport) {
    println!("Scenario completed successfully!");
    println!("Report ID: {}", report.scenario_id);
    println!("Variants generated: {}", report.variants.len());
    for variant in &report.variants {
        println!(" - Variant: {}", variant.variant_id);
        if let Some(path) = &variant.mutated_pdf {
            println!("   Path: {}", path.display());
        }
        if let Some(hash) = &variant.variant_hash {
            println!("   Hash: {}", hash);
        }
    }
}

fn ensure_demo_pdf(path: &PathBuf) {
    if !path.exists() {
        println!("Creating demo PDF at {}", path.display());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        
        // Create a minimal valid PDF using lopdf
        let mut doc = lopdf::Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        });
        let content = lopdf::content::Content {
            operations: vec![
                lopdf::content::Operation::new("BT", vec![]),
                lopdf::content::Operation::new("Tf", vec!["F1".into(), 12.into()]),
                lopdf::content::Operation::new("Td", vec![100.into(), 700.into()]),
                lopdf::content::Operation::new("Tj", vec![lopdf::Object::string_literal("SuperpoweredCV Demo Resume")]),
                lopdf::content::Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(lopdf::Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Contents" => content_id,
            "Resources" => resources_id,
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.save(path).unwrap();
    }
}
