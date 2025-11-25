use serde::{Deserialize, Serialize};

/// Contact information for the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactInfo {
    /// Email address.
    pub email: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// List of personal or professional websites.
    pub websites: Vec<String>,
    /// LinkedIn profile URL.
    pub linkedin: Option<String>,
    /// GitHub profile URL.
    pub github: Option<String>,
    /// Physical location (City, Country).
    pub location: Option<String>,
}

/// Represents a work experience entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Experience {
    /// Job title.
    pub title: String,
    /// Company name.
    pub company: String,
    /// Location of the job.
    pub location: Option<String>,
    /// Start date (e.g., "Jan 2020").
    pub start_date: Option<String>,
    /// End date (e.g., "Present", "Dec 2022").
    pub end_date: Option<String>,
    /// A brief summary of the role.
    pub summary: Option<String>,
    /// List of bullet points describing achievements and responsibilities.
    pub bullets: Vec<String>,
    /// List of technologies used in this role.
    pub tech_stack: Vec<String>,
}

/// Represents an educational background.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Education {
    /// Name of the institution.
    pub institution: String,
    /// Degree obtained (e.g., "B.S. Computer Science").
    pub degree: Option<String>,
    /// Field of study.
    pub field_of_study: Option<String>,
    /// Start date.
    pub start_date: Option<String>,
    /// End date or expected graduation date.
    pub end_date: Option<String>,
    /// Additional details or summary.
    pub summary: Option<String>,
}

/// Represents a specific skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skill {
    /// Name of the skill (e.g., "Rust", "Docker").
    pub name: String,
    /// Category of the skill (e.g., "Languages", "DevOps").
    pub category: Option<String>,
    /// Proficiency level (e.g., "Expert", "Intermediate").
    pub proficiency: Option<String>,
}

/// Represents a project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    /// Name of the project.
    pub name: String,
    /// Description of the project.
    pub description: Option<String>,
    /// Link to the project (e.g., GitHub repo, live demo).
    pub link: Option<String>,
    /// Technologies used in the project.
    pub technologies: Vec<String>,
}

/// Represents a professional certification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Certification {
    /// Name of the certification.
    pub name: String,
    /// Issuing organization.
    pub issuer: Option<String>,
    /// Date of issue.
    pub date: Option<String>,
    /// Link to the certificate or verification.
    pub link: Option<String>,
}

/// Represents a publication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Publication {
    /// Title of the publication.
    pub title: String,
    /// Publisher or conference name.
    pub publisher: Option<String>,
    /// Date of publication.
    pub date: Option<String>,
    /// Link to the publication.
    pub link: Option<String>,
    /// Summary or abstract.
    pub summary: Option<String>,
}

/// Represents a volunteering experience.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volunteering {
    /// Organization name.
    pub organization: String,
    /// Role or title.
    pub role: String,
    /// Start date.
    pub start_date: Option<String>,
    /// End date.
    pub end_date: Option<String>,
    /// Summary of activities.
    pub summary: Option<String>,
}

/// Represents a language proficiency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Language {
    /// Language name.
    pub name: String,
    /// Proficiency level (e.g., "Native", "Fluent", "B2").
    pub proficiency: Option<String>,
}

/// Metadata for the profile, including audit tags and visibility settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileMeta {
    /// Tags for auditing or categorization.
    pub audit_tags: Vec<String>,
    /// Visibility setting (e.g., "Public", "Private").
    pub visibility: Option<String>,
}

/// Seniority level for AI/ATS targeting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Seniority {
    Junior,
    Mid,
    Senior,
    Lead,
    Principal,
}

/// Metadata specifically for AI/ATS optimization and targeting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiAtsMetadata {
    /// Target roles for this profile.
    pub role_targets: Vec<String>,
    /// Target seniority level.
    pub seniority: Option<Seniority>,
    /// Target domains or industries.
    pub domains: Vec<String>,
    /// Taxonomy of skills for matching.
    pub skills_taxonomy: Vec<String>,
    /// Keywords to emphasize.
    pub keywords: Vec<String>,
    /// Notes for a human reviewer (if any).
    pub notes_for_human_reviewer: Option<String>,
}

/// The main user profile structure, aggregating all personal and professional data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    /// Unique identifier for the profile.
    pub id: String,
    /// Full name.
    pub name: String,
    /// Professional headline.
    pub headline: Option<String>,
    /// Location string.
    pub location: Option<String>,
    /// Professional summary or bio.
    pub summary: Option<String>,
    /// Contact information.
    pub contact: ContactInfo,
    /// List of work experiences.
    pub experience: Vec<Experience>,
    /// List of educational backgrounds.
    pub education: Vec<Education>,
    /// List of skills.
    pub skills: Vec<Skill>,
    /// List of projects.
    pub projects: Vec<Project>,
    /// List of certifications.
    pub certifications: Vec<Certification>,
    /// List of publications.
    pub publications: Vec<Publication>,
    /// List of volunteering experiences.
    pub volunteering: Vec<Volunteering>,
    /// List of languages.
    pub languages: Vec<Language>,
    /// General metadata.
    pub meta: Option<ProfileMeta>,
    /// AI/ATS specific metadata.
    pub ai_metadata: Option<AiAtsMetadata>,
}
