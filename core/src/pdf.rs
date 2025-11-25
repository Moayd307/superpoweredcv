use crate::analysis::ProfileConfig;
use crate::templates::AnalysisTemplate;
use crate::Result;
use lopdf::{Document, Object, StringFormat, dictionary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Request to mutate a PDF with a specific analysis profile and template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMutationRequest {
    /// Path to the base PDF file.
    pub base_pdf: PathBuf,
    /// Configuration for the analysis profile.
    pub profile: ProfileConfig,
    /// The analysis template to use.
    pub template: AnalysisTemplate,
    /// Optional ID for the variant.
    pub variant_id: Option<String>,
}

/// Result of a PDF mutation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMutationResult {
    /// Unique ID of the generated variant.
    pub variant_id: String,
    /// Path to the mutated PDF file.
    pub mutated_pdf: PathBuf,
    /// Hash of the mutated PDF content.
    pub variant_hash: Option<String>,
    /// Notes or logs from the mutation process.
    pub notes: Vec<String>,
}

/// Trait for components that can mutate PDFs.
pub trait PdfMutator {
    /// Mutates a PDF based on the request.
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult>;
}

/// A real PDF mutator that uses lopdf to modify PDF files.
pub struct RealPdfMutator {
    /// Directory where mutated PDFs will be saved.
    pub output_dir: PathBuf,
}

impl RealPdfMutator {
    /// Creates a new `RealPdfMutator` with the specified output directory.
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        RealPdfMutator {
            output_dir: output_dir.into(),
        }
    }
}

impl PdfMutator for RealPdfMutator {
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult> {
        fs::create_dir_all(&self.output_dir)?;

        let variant_id = request
            .variant_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let file_name = format!("{}.pdf", variant_id);
        let output_path = self.output_dir.join(file_name);

        // Load the base PDF
        let mut doc = Document::load(&request.base_pdf)
            .map_err(|e| crate::AnalysisError::PdfError(format!("Failed to load PDF: {}", e)))?;

        // Apply injection strategy
        // For this initial implementation, we use Metadata Injection.
        // We inject the template content into a custom metadata field.
        // TODO: Implement other strategies:
        // - Invisible text injection (white text on white background)
        // - Font substitution (custom font with swapped glyphs)
        // - Content stream manipulation (appending text to page content)
        
        // Ensure Info dictionary exists or get reference to it
        let info_id = match doc.trailer.get(b"Info").ok().and_then(|obj| obj.as_reference().ok()) {
            Some(id) => id,
            None => {
                let info_id = doc.add_object(dictionary! {});
                doc.trailer.set("Info", info_id);
                info_id
            }
        };

        if let Ok(info) = doc.get_object_mut(info_id) {
            if let Object::Dictionary(dict) = info {
                // Inject content into "CustomInjection" field
                // We use StringFormat::Literal to ensure it's treated as text
                dict.set(
                    "CustomInjection", 
                    Object::String(request.template.text_template.clone().into(), StringFormat::Literal)
                );
                
                // Also update Producer to mark it as modified by our tool
                dict.set(
                    "Producer",
                    Object::String("SuperpoweredCV Analysis Tool".into(), StringFormat::Literal)
                );
            }
        }

        // Save the mutated PDF
        // We use save_to to write to the output path
        let mut file = fs::File::create(&output_path)?;
        doc.save_to(&mut file)
            .map_err(|e| crate::AnalysisError::PdfError(format!("Failed to save PDF: {}", e)))?;

        // Calculate hash of the new file
        let mut hasher = Sha256::new();
        let content = fs::read(&output_path)?;
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        Ok(PdfMutationResult {
            variant_id,
            mutated_pdf: output_path,
            variant_hash: Some(hash),
            notes: vec![
                "RealPdfMutator: Applied metadata injection".into(),
                format!("Injected content length: {}", request.template.text_template.len()),
            ],
        })
    }
}

/// A placeholder mutator that writes a small marker file containing mutation
/// metadata. This gives downstream code a tangible artifact (with hash) without
/// requiring a PDF stack during early development.
pub struct StubPdfMutator {
    /// Directory where mutated PDFs (stubs) will be saved.
    pub output_dir: PathBuf,
}

impl StubPdfMutator {
    /// Creates a new `StubPdfMutator` with the specified output directory.
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

        let file_name = format!("{}.pdf", variant_id);
        let output_path = self.output_dir.join(file_name);

        // In a real implementation, this would apply the injection.
        // Here we just copy the base PDF if it exists, or create a dummy one.
        if request.base_pdf.exists() {
            fs::copy(&request.base_pdf, &output_path)?;
        } else {
            // Create a dummy PDF file for testing
            fs::write(&output_path, b"%PDF-1.4\n%Dummy PDF content for testing")?;
        }

        // Calculate a simple hash of the output file
        let mut hasher = Sha256::new();
        let content = fs::read(&output_path)?;
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        Ok(PdfMutationResult {
            variant_id,
            mutated_pdf: output_path,
            variant_hash: Some(hash),
            notes: vec![
                "Stub mutator: copied base PDF (or created dummy)".into(),
                format!("Applied profile: {:?}", request.profile),
            ],
        })
    }
}
