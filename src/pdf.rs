use crate::red_team::ProfileConfig;
use crate::templates::InjectionTemplate;
use crate::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMutationRequest {
    pub base_pdf: PathBuf,
    pub profile: ProfileConfig,
    pub template: InjectionTemplate,
    pub variant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMutationResult {
    pub variant_id: String,
    pub mutated_pdf: PathBuf,
    pub variant_hash: Option<String>,
    pub notes: Vec<String>,
}

pub trait PdfMutator {
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult>;
}

/// A placeholder mutator that writes a small marker file containing mutation
/// metadata. This gives downstream code a tangible artifact (with hash) without
/// requiring a PDF stack during early development.
pub struct StubPdfMutator {
    pub output_dir: PathBuf,
}

impl StubPdfMutator {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        StubPdfMutator {
            output_dir: output_dir.into(),
        }
    }
}

impl PdfMutator for StubPdfMutator {
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult> {
        fs::create_dir_all(&self.output_dir)?;

        let variant_id = request
            .variant_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let filename = format!("{variant_id}.txt");
        let output_path = self.output_dir.join(filename);
        let contents = format!(
            "variant_id: {variant_id}\nprofile: {}\ntemplate: {}\nbase_pdf: {}\n",
            request.profile.id(),
            request.template.id,
            request.base_pdf.display()
        );

        fs::write(&output_path, contents.as_bytes())?;
        let variant_hash = Some(hash_bytes(&contents));

        Ok(PdfMutationResult {
            variant_id,
            mutated_pdf: output_path,
            variant_hash,
            notes: vec!["stub mutation created placeholder artifact".into()],
        })
    }
}

pub fn hash_bytes(bytes: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes.as_ref());
    let digest = hasher.finalize();
    hex::encode(digest)
}
