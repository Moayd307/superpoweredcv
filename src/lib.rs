pub mod pipeline;
pub mod profile;
pub mod red_team;
pub mod templates;

pub type Result<T> = std::result::Result<T, RedTeamError>;

#[derive(Debug, thiserror::Error)]
pub enum RedTeamError {
    #[error("template `{0}` not found")]
    MissingTemplate(String),
    #[error("profile `{0}` not supported")]
    UnsupportedProfile(String),
    #[error("invalid scenario: {0}")]
    InvalidScenario(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
