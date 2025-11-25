use crate::pdf::{PdfMutationRequest, PdfMutator, StubPdfMutator};
use crate::pipeline::{LoggingConfig, MetricSpec, PipelineConfig};
use crate::templates::InjectionTemplate;
use crate::{Result, SimulationError};
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

/// Configuration for the injection profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileConfig {
    /// Visible block of meta-instructions.
    VisibleMetaBlock {
        /// Position of the block.
        position: InjectionPosition,
        /// Intensity of the instructions.
        intensity: Intensity,
    },
    /// Low-visibility block (small font, low contrast).
    LowVisibilityBlock {
        /// Minimum font size.
        font_size_min: u8,
        /// Maximum font size.
        font_size_max: u8,
        /// Color palette to use.
        color_profile: LowVisibilityPalette,
    },
    /// Text placed off the visible page area.
    OffpageLayer {
        /// Offset strategy.
        offset_strategy: OffpageOffset,
        /// Length of the text.
        length: Option<u32>,
    },
    /// Text hidden under other elements.
    UnderlayText,
    /// Injection into structural fields (metadata, tags).
    StructuralFields {
        /// List of targets.
        targets: Vec<StructuralTarget>,
    },
    /// Noise padding to confuse the model.
    PaddingNoise {
        /// Tokens of padding before the content.
        padding_tokens_before: Option<u32>,
        /// Tokens of padding after the content.
        padding_tokens_after: Option<u32>,
        /// Style of the padding.
        padding_style: PaddingStyle,
    },
    /// Injection of a job advertisement.
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
        }
    }
}

/// Plan for a single injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InjectionPlan {
    /// The profile configuration to use.
    pub profile: ProfileConfig,
    /// The ID of the template to use.
    pub template_id: String,
}

/// Defines a complete injection scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionScenario {
    /// Unique ID for the scenario.
    pub scenario_id: String,
    /// Path to the base PDF file.
    pub base_pdf: PathBuf,
    /// List of injections to perform.
    pub injections: Vec<InjectionPlan>,
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

/// The main engine for running Simulation scenarios.
pub struct SimulationEngine {
    templates: HashMap<String, InjectionTemplate>,
}

impl SimulationEngine {
    /// Creates a new `SimulationEngine` with the provided templates.
    pub fn new(templates: impl IntoIterator<Item = InjectionTemplate>) -> Self {
        let map = templates
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect::<HashMap<_, _>>();
        SimulationEngine { templates: map }
    }

    fn template(&self, id: &str) -> Result<&InjectionTemplate> {
        self.templates
            .get(id)
            .ok_or_else(|| SimulationError::MissingTemplate(id.to_string()))
    }

    fn build_variant_id(profile: &ProfileConfig, template: &InjectionTemplate) -> String {
        format!("{}_{}", profile.id(), template.id.replace('.', "_"))
    }

    /// Runs a scenario with a specific mutator and pipeline executor.
    pub fn run_with(
        &self,
        scenario: &InjectionScenario,
        mutator: &dyn PdfMutator,
        pipeline: &dyn PipelineExecutor,
    ) -> Result<ScenarioReport> {
        if scenario.injections.is_empty() {
            return Err(SimulationError::InvalidScenario(
                "scenario requires at least one injection".into(),
            ));
        }

        let mut impacts = Vec::new();
        for injection in &scenario.injections {
            let template = self.template(&injection.template_id)?;
            let variant_id = Self::build_variant_id(&injection.profile, template);

            let mutation = mutator.mutate(PdfMutationRequest {
                base_pdf: scenario.base_pdf.clone(),
                profile: injection.profile.clone(),
                template: template.clone(),
                variant_id: Some(variant_id.clone()),
            })?;

            let variant = PdfVariant {
                variant_id: mutation.variant_id.clone(),
                profiles: vec![injection.profile.id().to_string()],
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

    /// Runs a scenario using the default stub mutator and no-op pipeline.
    pub fn run_scenario(&self, scenario: &InjectionScenario) -> Result<ScenarioReport> {
        let mutator = StubPdfMutator::new("target/variants");
        let pipeline = NoopPipelineExecutor;
        self.run_with(scenario, &mutator, &pipeline)
    }
}

/// Trait for executing the evaluation pipeline.
pub trait PipelineExecutor {
    /// Evaluates a PDF variant against the scenario.
    fn evaluate(
        &self,
        variant: PdfVariant,
        scenario: &InjectionScenario,
    ) -> Result<VariantImpact>;
}

/// Placeholder pipeline executor that leaves scoring/classification empty but
/// threads through artifact metadata.
pub struct NoopPipelineExecutor;

impl PipelineExecutor for NoopPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        _scenario: &InjectionScenario,
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
