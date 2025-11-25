use crate::pdf::{PdfMutationRequest, PdfMutator, PdfMutationResult, StubPdfMutator};
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
    pub watermark_applied: bool,
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

    pub fn generate_variants(&self, scenario: &InjectionScenario) -> Result<Vec<PdfVariant>> {
        if scenario.injections.is_empty() {
            return Err(RedTeamError::InvalidScenario(
                "scenario requires at least one injection".into(),
            ));
        }

        let mut variants = Vec::new();
        for injection in &scenario.injections {
            let template = self
                .templates
                .get(&injection.template_id)
                .ok_or_else(|| RedTeamError::MissingTemplate(injection.template_id.clone()))?;

            let variant_id = format!(
                "{}_{}",
                injection.profile.id(),
                template.id.replace('.', "_")
            );

            variants.push(PdfVariant {
                variant_id,
                profiles: vec![injection.profile.id().to_string()],
                templates: vec![template.id.clone()],
                base_pdf: scenario.base_pdf.clone(),
                mutated_pdf: None,
            });
        }

        Ok(variants)
    }

    pub fn run_scenario(&self, scenario: &InjectionScenario) -> Result<ScenarioReport> {
        let variants = self.generate_variants(scenario)?;

        // Placeholder to show the structure; PDF mutation and pipeline calls are
        // implemented by downstream services.
        let impacts = variants
            .into_iter()
            .map(|variant| VariantImpact {
                variant_id: variant.variant_id.clone(),
                profiles: variant.profiles.clone(),
                templates: variant.templates.clone(),
                score_before: None,
                score_after: None,
                classification_before: None,
                classification_after: None,
                llm_response_sample: None,
            })
            .collect();

        Ok(ScenarioReport {
            scenario_id: scenario.scenario_id.clone(),
            target: None,
            variants: impacts,
        })
    }
}
