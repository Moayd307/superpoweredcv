use crate::pdf::{PdfMutationRequest, PdfMutator, StubPdfMutator};
use crate::pipeline::{LoggingConfig, MetricSpec, PipelineConfig};
use crate::templates::InjectionTemplate;
use crate::{Result, RedTeamError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InjectionPosition {
    Header,
    Footer,
    Section(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Intensity {
    Soft,
    Medium,
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LowVisibilityPalette {
    Gray,
    LightBlue,
    OffWhite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OffpageOffset {
    BottomClip,
    RightClip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuralTarget {
    AltText,
    PdfTag,
    XmpMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaddingStyle {
    ResumeLike,
    JobRelated,
    Lorem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAdSource {
    File,
    Inline,
    CacheId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAdPlacement {
    Front,
    Back,
    AfterSummary,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileConfig {
    VisibleMetaBlock {
        position: InjectionPosition,
        intensity: Intensity,
    },
    LowVisibilityBlock {
        font_size_min: u8,
        font_size_max: u8,
        color_profile: LowVisibilityPalette,
    },
    OffpageLayer {
        offset_strategy: OffpageOffset,
        length: Option<u32>,
    },
    UnderlayText,
    StructuralFields {
        targets: Vec<StructuralTarget>,
    },
    PaddingNoise {
        padding_tokens_before: Option<u32>,
        padding_tokens_after: Option<u32>,
        padding_style: PaddingStyle,
    },
    InlineJobAd {
        job_ad_source: JobAdSource,
        placement: JobAdPlacement,
        ad_excerpt_ratio: f32,
    },
}

impl ProfileConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InjectionPlan {
    pub profile: ProfileConfig,
    pub template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionScenario {
    pub scenario_id: String,
    pub base_pdf: PathBuf,
    pub injections: Vec<InjectionPlan>,
    pub pipeline: PipelineConfig,
    pub metrics: Vec<MetricSpec>,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdfVariant {
    pub variant_id: String,
    pub profiles: Vec<String>,
    pub templates: Vec<String>,
    pub base_pdf: PathBuf,
    pub mutated_pdf: Option<PathBuf>,
    pub variant_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantImpact {
    pub variant_id: String,
    pub score_before: Option<f64>,
    pub score_after: Option<f64>,
    pub classification_before: Option<String>,
    pub classification_after: Option<String>,
    pub llm_response_sample: Option<String>,
    pub profiles: Vec<String>,
    pub templates: Vec<String>,
    pub mutated_pdf: Option<PathBuf>,
    pub variant_hash: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioReport {
    pub scenario_id: String,
    pub target: Option<String>,
    pub variants: Vec<VariantImpact>,
}

pub struct RedTeamEngine {
    templates: HashMap<String, InjectionTemplate>,
}

impl RedTeamEngine {
    pub fn new(templates: impl IntoIterator<Item = InjectionTemplate>) -> Self {
        let map = templates
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect::<HashMap<_, _>>();
        RedTeamEngine { templates: map }
    }

    fn template(&self, id: &str) -> Result<&InjectionTemplate> {
        self.templates
            .get(id)
            .ok_or_else(|| RedTeamError::MissingTemplate(id.to_string()))
    }

    fn build_variant_id(profile: &ProfileConfig, template: &InjectionTemplate) -> String {
        format!("{}_{}", profile.id(), template.id.replace('.', "_"))
    }

    pub fn run_with(
        &self,
        scenario: &InjectionScenario,
        mutator: &dyn PdfMutator,
        pipeline: &dyn PipelineExecutor,
    ) -> Result<ScenarioReport> {
        if scenario.injections.is_empty() {
            return Err(RedTeamError::InvalidScenario(
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

    pub fn run_scenario(&self, scenario: &InjectionScenario) -> Result<ScenarioReport> {
        let mutator = StubPdfMutator::new("target/variants");
        let pipeline = NoopPipelineExecutor;
        self.run_with(scenario, &mutator, &pipeline)
    }
}

pub trait PipelineExecutor {
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
