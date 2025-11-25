use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineType {
    HttpLlm {
        endpoint: String,
        prompt_template: Option<String>,
    },
    LocalPrompt {
        model: Option<String>,
        prompt_template: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineConfig {
    pub pipeline_type: PipelineType,
    pub allow_aggressive_override: bool,
    pub target: Option<String>,
}

impl PipelineConfig {
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricType {
    NumericDiff,
    LabelChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricSpec {
    pub name: String,
    pub metric_type: MetricType,
    pub baseline: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoggingConfig {
    pub capture: Vec<LogField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogField {
    RawLlmResponse,
    ExtractedText,
    PdfVariantHash,
}
