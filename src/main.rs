use superpoweredcv::pipeline::{LoggingConfig, LogField, MetricSpec, MetricType, PipelineConfig, PipelineType};
use superpoweredcv::red_team::{
    InjectionPlan, InjectionScenario, Intensity, InjectionPosition, JobAdPlacement, JobAdSource,
    PaddingStyle, ProfileConfig, RedTeamEngine,
};
use superpoweredcv::templates::default_templates;
use std::path::PathBuf;

fn main() {
    let scenario = InjectionScenario {
        scenario_id: "ats_pdf_injection_smoke".into(),
        base_pdf: PathBuf::from("examples/clean_resume.pdf"),
        injections: vec![
            InjectionPlan {
                profile: ProfileConfig::VisibleMetaBlock {
                    position: InjectionPosition::Footer,
                    intensity: Intensity::Soft,
                },
                template_id: "soft_bias".into(),
            },
            InjectionPlan {
                profile: ProfileConfig::PaddingNoise {
                    padding_tokens_before: Some(256),
                    padding_tokens_after: Some(256),
                    padding_style: PaddingStyle::JobRelated,
                },
                template_id: "aggressive_override".into(),
            },
            InjectionPlan {
                profile: ProfileConfig::InlineJobAd {
                    job_ad_source: JobAdSource::Inline,
                    placement: JobAdPlacement::AfterSummary,
                    ad_excerpt_ratio: 0.5,
                },
                template_id: "override_conflict".into(),
            },
        ],
        pipeline: PipelineConfig {
            pipeline_type: PipelineType::HttpLlm {
                endpoint: "https://example-ats-llm/api/score".into(),
                prompt_template: Some("prompts/ats_prompt.txt".into()),
            },
            target: Some("candidate_scoring_service_v2".into()),
        },
        metrics: vec![
            MetricSpec {
                name: "score_shift".into(),
                metric_type: MetricType::NumericDiff,
                baseline: Some(0.0),
            },
            MetricSpec {
                name: "classification_change".into(),
                metric_type: MetricType::LabelChange,
                baseline: None,
            },
        ],
        logging: Some(LoggingConfig {
            capture: vec![
                LogField::RawLlmResponse,
                LogField::ExtractedText,
                LogField::PdfVariantHash,
            ],
        }),
    };

    let engine = RedTeamEngine::new(default_templates());
    match engine.run_scenario(&scenario) {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("valid serialization")
            );
        }
        Err(err) => eprintln!("Scenario failed: {err}"),
    }
}
