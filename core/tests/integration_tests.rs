use superpoweredcv::pdf::{RealPdfMutator, PdfMutator, PdfMutationRequest};
use superpoweredcv::analysis::{ProfileConfig, InjectionPosition, Intensity, LowVisibilityPalette};
use superpoweredcv::templates::{AnalysisTemplate, TemplateSeverity, TemplateStyle, ControlType};
use superpoweredcv::pdf_utils;
use std::path::PathBuf;
use std::fs;

#[test]
fn test_pdf_mutation_visible_block() {
    let output_dir = PathBuf::from("target/test_output");
    fs::create_dir_all(&output_dir).unwrap();
    
    // Create a dummy base PDF
    let base_pdf_path = output_dir.join("base.pdf");
    let mut doc = pdf_utils::create_blank_pdf();
    doc.save(&base_pdf_path).unwrap();

    let mutator = RealPdfMutator::new(&output_dir);
    
    let request = PdfMutationRequest {
        base_pdf: base_pdf_path,
        profile: ProfileConfig::VisibleMetaBlock {
            position: InjectionPosition::Header,
            intensity: Intensity::Medium,
            content: Default::default(),
        },
        template: AnalysisTemplate {
            id: "test_template".to_string(),
            severity: TemplateSeverity::Low,
            goal: "Test Goal".to_string(),
            style: TemplateStyle::Subtle,
            control: ControlType::Plain,
            text_template: "This is a test injection.".to_string(),
            phrases: vec![],
            generation_type: Default::default(),
            job_description: None,
        },
        variant_id: Some("test_variant_visible".to_string()),
    };

    let result = mutator.mutate(request).unwrap();
    
    assert!(result.mutated_pdf.exists());
    assert!(result.variant_hash.is_some());
    assert!(result.notes.iter().any(|n| n.contains("Injected visible block")));
}

#[test]
fn test_pdf_mutation_low_visibility() {
    let output_dir = PathBuf::from("target/test_output");
    fs::create_dir_all(&output_dir).unwrap();
    
    let base_pdf_path = output_dir.join("base_low.pdf");
    let mut doc = pdf_utils::create_blank_pdf();
    doc.save(&base_pdf_path).unwrap();

    let mutator = RealPdfMutator::new(&output_dir);
    
    let request = PdfMutationRequest {
        base_pdf: base_pdf_path,
        profile: ProfileConfig::LowVisibilityBlock {
            font_size_min: 1,
            font_size_max: 1,
            color_profile: LowVisibilityPalette::Gray,
            content: Default::default(),
        },
        template: AnalysisTemplate {
            id: "test_template".to_string(),
            severity: TemplateSeverity::Low,
            goal: "Test Goal".to_string(),
            style: TemplateStyle::Subtle,
            control: ControlType::Plain,
            text_template: "This is a hidden test injection.".to_string(),
            phrases: vec![],
            generation_type: Default::default(),
            job_description: None,
        },
        variant_id: Some("test_variant_low".to_string()),
    };

    let result = mutator.mutate(request).unwrap();
    
    assert!(result.mutated_pdf.exists());
    assert!(result.notes.iter().any(|n| n.contains("Injected low visibility block")));
}
