use crate::pdf::{PdfMutationRequest, PdfMutator, RealPdfMutator};
use crate::pipeline::{LoggingConfig, MetricSpec, PipelineConfig, PipelineType};
use crate::templates::{AnalysisTemplate, GenerationType};
use crate::{Result, AnalysisError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Defines where the injection should be placed in the document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InjectionPosition {
    /// Place in the header.
    Header,
    /// Place in the footer.
    Footer,
    /// Place in a specific named section.
    Section(String),
}

/// Defines the intensity of the injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Intensity {
    /// Soft intensity.
    Soft,
    /// Medium intensity.
    Medium,
    /// Aggressive intensity.
    Aggressive,
    /// Custom intensity (uses provided content directly).
    Custom,
}

/// Palette for low-visibility text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LowVisibilityPalette {
    /// Gray color.
    Gray,
    /// Light blue color.
    LightBlue,
    /// Off-white color.
    OffWhite,
}

/// Strategy for placing text off-page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OffpageOffset {
    /// Clip at the bottom of the page.
    BottomClip,
    /// Clip at the right of the page.
    RightClip,
}

/// Target for structural injections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuralTarget {
    /// Inject into Alt Text.
    AltText,
    /// Inject into PDF Tags.
    PdfTag,
    /// Inject into XMP Metadata.
    XmpMetadata,
}

/// Style of padding noise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaddingStyle {
    /// Padding that looks like resume content.
    ResumeLike,
    /// Padding related to the job description.
    JobRelated,
    /// Lorem ipsum padding.
    Lorem,
}

/// Source of the job advertisement text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAdSource {
    /// Load from a file.
    File,
    /// Provided inline.
    Inline,
    /// Load from a cache ID.
    CacheId,
}

/// Placement of the job ad injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAdPlacement {
    /// Place at the front of the document.
    Front,
    /// Place at the back of the document.
    Back,
    /// Place after the summary section.
    AfterSummary,
    /// Custom placement.
    Custom,
}

/// Content configuration for the injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InjectionContent {
    /// List of phrases to inject.
    #[serde(default)]
    pub phrases: Vec<String>,
    /// How the content is generated.
    #[serde(default)]
    pub generation_type: GenerationType,
    /// Job description for ad-targeted pollution.
    #[serde(default)]
    pub job_description: Option<String>,
}

impl Default for InjectionContent {
    fn default() -> Self {
        Self {
            phrases: vec![],
            generation_type: GenerationType::Static,
            job_description: None,
        }
    }
}

/// Configuration for the injection profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileConfig {
    /// Visible block of meta-instructions.
    VisibleMetaBlock {
        /// Position of the block.
        position: InjectionPosition,
        /// Intensity of the instructions.
        intensity: Intensity,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Low-visibility block (small font, low contrast).
    LowVisibilityBlock {
        /// Minimum font size.
        font_size_min: u8,
        /// Maximum font size.
        font_size_max: u8,
        /// Color palette to use.
        color_profile: LowVisibilityPalette,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Text placed off the visible page area.
    OffpageLayer {
        /// Offset strategy.
        offset_strategy: OffpageOffset,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Tracking pixel injection (URL link).
    TrackingPixel {
        /// The URL to track.
        url: String,
    },
    /// Code injection (JavaScript).
    CodeInjection {
        /// The JavaScript payload.
        payload: String,
    },
    /// Text hidden behind other content.
    UnderlayText,
    /// Injection into structural fields (metadata, tags).
    StructuralFields {
        /// Targets for injection.
        targets: Vec<StructuralTarget>,
    },
    /// Noise padding around content.
    PaddingNoise {
        /// Number of tokens before.
        padding_tokens_before: usize,
        /// Number of tokens after.
        padding_tokens_after: usize,
        /// Style of padding.
        padding_style: PaddingStyle,
    },
    /// Inline job advertisement injection.
    InlineJobAd {
        /// Source of the job ad.
        job_ad_source: JobAdSource,
        /// Placement of the ad.
        placement: JobAdPlacement,
        /// Ratio of the ad to include.
        ad_excerpt_ratio: f32,
    },
}

impl ProfileConfig {
    /// Returns the unique ID of the profile configuration type.
    pub fn id(&self) -> &'static str {
        match self {
            ProfileConfig::VisibleMetaBlock { .. } => "pdf.visible_meta_block",
            ProfileConfig::LowVisibilityBlock { .. } => "pdf.low_visibility_block",
            ProfileConfig::OffpageLayer { .. } => "pdf.offpage_layer",
            ProfileConfig::UnderlayText => "pdf.underlay_text",
            ProfileConfig::StructuralFields { .. } => "pdf.structural_fields",
            ProfileConfig::PaddingNoise { .. } => "pdf.padding_noise",
            ProfileConfig::InlineJobAd { .. } => "pdf.inline_job_ad",
            ProfileConfig::TrackingPixel { .. } => "pdf.tracking_pixel",
            ProfileConfig::CodeInjection { .. } => "pdf.code_injection",
        }
    }
}

/// Plan for a single analysis step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisPlan {
    /// The profile configuration to use.
    pub profile: ProfileConfig,
    /// The ID of the template to use.
    pub template_id: String,
}

/// Defines a complete analysis scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisScenario {
    /// Unique ID for the scenario.
    pub scenario_id: String,
    /// Path to the base PDF file.
    pub base_pdf: PathBuf,
    /// List of analysis plans to perform.
    pub plans: Vec<AnalysisPlan>,
    /// Configuration for the evaluation pipeline.
    pub pipeline: PipelineConfig,
    /// List of metrics to track.
    pub metrics: Vec<MetricSpec>,
    /// Logging configuration.
    pub logging: Option<LoggingConfig>,
}

/// Represents a generated PDF variant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdfVariant {
    /// Unique ID of the variant.
    pub variant_id: String,
    /// List of profile IDs applied.
    pub profiles: Vec<String>,
    /// List of template IDs applied.
    pub templates: Vec<String>,
    /// Path to the base PDF.
    pub base_pdf: PathBuf,
    /// Path to the mutated PDF.
    pub mutated_pdf: Option<PathBuf>,
    /// Hash of the variant.
    pub variant_hash: Option<String>,
}

/// The impact of a variant on the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantImpact {
    /// ID of the variant.
    pub variant_id: String,
    /// Score before injection.
    pub score_before: Option<f64>,
    /// Score after injection.
    pub score_after: Option<f64>,
    /// Classification label before injection.
    pub classification_before: Option<String>,
    /// Classification label after injection.
    pub classification_after: Option<String>,
    /// Sample response from the LLM.
    pub llm_response_sample: Option<String>,
    /// Profiles applied.
    pub profiles: Vec<String>,
    /// Templates applied.
    pub templates: Vec<String>,
    /// Path to the mutated PDF.
    pub mutated_pdf: Option<PathBuf>,
    /// Hash of the variant.
    pub variant_hash: Option<String>,
    /// Notes or logs.
    pub notes: Vec<String>,
}

/// Report for a full scenario execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioReport {
    /// ID of the scenario.
    pub scenario_id: String,
    /// Target pipeline name.
    pub target: Option<String>,
    /// List of impacts for each variant.
    pub variants: Vec<VariantImpact>,
}

/// The main engine for running Analysis scenarios.
pub struct AnalysisEngine {
    templates: HashMap<String, AnalysisTemplate>,
}

impl AnalysisEngine {
    /// Creates a new `AnalysisEngine` with the provided templates.
    pub fn new(templates: impl IntoIterator<Item = AnalysisTemplate>) -> Self {
        let map = templates
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect::<HashMap<_, _>>();
        AnalysisEngine { templates: map }
    }

    fn template(&self, id: &str) -> Result<&AnalysisTemplate> {
        self.templates
            .get(id)
            .ok_or_else(|| AnalysisError::MissingTemplate(id.to_string()))
    }

    fn build_variant_id(profile: &ProfileConfig, template: &AnalysisTemplate) -> String {
        format!("{}_{}", profile.id(), template.id.replace('.', "_"))
    }

    /// Runs a scenario with a specific mutator and pipeline executor.
    pub fn run_with(
        &self,
        scenario: &AnalysisScenario,
        mutator: &dyn PdfMutator,
        pipeline: &dyn PipelineExecutor,
    ) -> Result<ScenarioReport> {
        if scenario.plans.is_empty() {
            return Err(AnalysisError::InvalidScenario(
                "scenario requires at least one plan".into(),
            ));
        }

        let mut impacts = Vec::new();
        for plan in &scenario.plans {
            let template = self.template(&plan.template_id)?;
            let variant_id = Self::build_variant_id(&plan.profile, template);

            let mutation = mutator.mutate(PdfMutationRequest {
                base_pdf: scenario.base_pdf.clone(),
                profiles: vec![plan.profile.clone()],
                template: template.clone(),
                variant_id: Some(variant_id.clone()),
            })?;

            let variant = PdfVariant {
                variant_id: mutation.variant_id.clone(),
                profiles: vec![plan.profile.id().to_string()],
                templates: vec![template.id.clone()],
                base_pdf: scenario.base_pdf.clone(),
                mutated_pdf: Some(mutation.mutated_pdf.clone()),
                variant_hash: mutation.variant_hash.clone(),
            };

            let mut impact = pipeline.evaluate(variant.clone(), scenario)?;
            if impact.mutated_pdf.is_none() {
                impact.mutated_pdf = variant.mutated_pdf.clone();
            }
            if impact.variant_hash.is_none() {
                impact.variant_hash = variant.variant_hash.clone();
            }
            if impact.profiles.is_empty() {
                impact.profiles = variant.profiles.clone();
            }
            if impact.templates.is_empty() {
                impact.templates = variant.templates.clone();
            }

            impacts.push(impact);
        }

        Ok(ScenarioReport {
            scenario_id: scenario.scenario_id.clone(),
            target: scenario.pipeline.target().map(|t| t.to_string()),
            variants: impacts,
        })
    }

    /// Runs a scenario using the real mutator and http pipeline executor.
    pub fn run_scenario(&self, scenario: &AnalysisScenario) -> Result<ScenarioReport> {
        let mutator = RealPdfMutator::new("target/variants");
        let pipeline = HttpPipelineExecutor::new();
        self.run_with(scenario, &mutator, &pipeline)
    }
}

/// Trait for executing the evaluation pipeline.
pub trait PipelineExecutor {
    /// Evaluates a PDF variant against the scenario.
    fn evaluate(
        &self,
        variant: PdfVariant,
        scenario: &AnalysisScenario,
    ) -> Result<VariantImpact>;
}

/// Placeholder pipeline executor that leaves scoring/classification empty but
/// threads through artifact metadata.
pub struct NoopPipelineExecutor;

impl PipelineExecutor for NoopPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        _scenario: &AnalysisScenario,
    ) -> Result<VariantImpact> {
        Ok(VariantImpact {
            variant_id: variant.variant_id,
            score_before: None,
            score_after: None,
            classification_before: None,
            classification_after: None,
            llm_response_sample: None,
            profiles: variant.profiles,
            templates: variant.templates,
            mutated_pdf: variant.mutated_pdf,
            variant_hash: variant.variant_hash,
            notes: vec!["pipeline execution skipped (noop executor)".into()],
        })
    }
}

/// Pipeline executor that sends requests to an HTTP endpoint.
pub struct HttpPipelineExecutor {
    client: reqwest::blocking::Client,
}

impl HttpPipelineExecutor {
    /// Creates a new HttpPipelineExecutor.
    pub fn new() -> Self {
        HttpPipelineExecutor {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl PipelineExecutor for HttpPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        scenario: &AnalysisScenario,
    ) -> Result<VariantImpact> {
        match &scenario.pipeline.pipeline_type {
            PipelineType::HttpLlm { endpoint, .. } => {
                // If the endpoint is the example one, skip execution to avoid errors
                if endpoint.contains("example-ats-llm") {
                     return Ok(VariantImpact {
                        variant_id: variant.variant_id,
                        score_before: None,
                        score_after: None,
                        classification_before: None,
                        classification_after: None,
                        llm_response_sample: None,
                        profiles: variant.profiles,
                        templates: variant.templates,
                        mutated_pdf: variant.mutated_pdf,
                        variant_hash: variant.variant_hash,
                        notes: vec!["HttpPipelineExecutor: Skipped example endpoint".into()],
                    });
                }

                // Prepare the request
                let file_path = variant.mutated_pdf.as_ref()
                    .ok_or_else(|| crate::AnalysisError::InvalidScenario("Missing mutated PDF path".into()))?;
                
                let form = reqwest::blocking::multipart::Form::new()
                    .file("file", file_path)
                    .map_err(|e| crate::AnalysisError::Io(e))?;

                let response = self.client.post(endpoint)
                    .multipart(form)
                    .send()
                    .map_err(|e| crate::AnalysisError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                let status = response.status();
                let text = response.text().unwrap_or_default();

                Ok(VariantImpact {
                    variant_id: variant.variant_id,
                    score_before: None,
                    score_after: None,
                    classification_before: None,
                    classification_after: None,
                    llm_response_sample: Some(text),
                    profiles: variant.profiles,
                    templates: variant.templates,
                    mutated_pdf: variant.mutated_pdf,
                    variant_hash: variant.variant_hash,
                    notes: vec![format!("HttpPipelineExecutor: POST {} -> {}", endpoint, status)],
                })
            }
            _ => {
                // Fallback to no-op
                 Ok(VariantImpact {
                    variant_id: variant.variant_id,
                    score_before: None,
                    score_after: None,
                    classification_before: None,
                    classification_after: None,
                    llm_response_sample: None,
                    profiles: variant.profiles,
                    templates: variant.templates,
                    mutated_pdf: variant.mutated_pdf,
                    variant_hash: variant.variant_hash,
                    notes: vec!["HttpPipelineExecutor: Unsupported pipeline type".into()],
                })
            }
        }
    }
}
