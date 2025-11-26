pub mod pipeline;
pub mod pdf;
pub mod pdf_utils;
pub mod profile;
pub mod analysis;
pub mod templates;
pub mod generator;
pub mod config;
pub mod llm;

/// A specialized result type for Analysis operations.
pub type Result<T> = std::result::Result<T, AnalysisError>;

/// Errors that can occur during Analysis operations.
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    /// The requested analysis template was not found in the engine's registry.
    #[error("template `{0}` not found")]
    MissingTemplate(String),
    /// The requested profile configuration is not supported by the current mutator.
    #[error("profile `{0}` not supported")]
    UnsupportedProfile(String),
    /// The scenario configuration is invalid (e.g., no plans specified).
    #[error("invalid scenario: {0}")]
    InvalidScenario(String),
    /// An I/O error occurred (e.g., file not found, permission denied).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A PDF processing error occurred.
    #[error("PDF error: {0}")]
    PdfError(String),
}
