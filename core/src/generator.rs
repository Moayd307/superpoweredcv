use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::Result;
use lopdf::{Document, Object, dictionary};
use lopdf::content::{Content, Operation};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapedProfile {
    pub name: String,
    pub headline: String,
    pub location: String,
    pub about: String,
    pub experience: Vec<ScrapedExperience>,
    pub education: Vec<ScrapedEducation>,
    pub skills: Vec<String>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapedExperience {
    pub title: String,
    pub company: String,
    pub date_range: String,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapedEducation {
    pub school: String,
    pub degree: String,
}

pub fn generate_pdf(profile: &ScrapedProfile, output: &Path) -> Result<()> {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut operations = Vec::new();
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec!["F1".into(), 14.into()]));
    operations.push(Operation::new("Td", vec![50.into(), 750.into()]));
    
    // Name
    operations.push(Operation::new("Tj", vec![Object::string_literal(format!("Name: {}", profile.name))]));
    operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-20)]));
    
    // Headline
    operations.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal(format!("Headline: {}", profile.headline))]));
    operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-20)]));

    // Location
    operations.push(Operation::new("Tj", vec![Object::string_literal(format!("Location: {}", profile.location))]));
    operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-30)]));

    // Experience Header
    operations.push(Operation::new("Tf", vec!["F1".into(), 14.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal("Experience")]));
    operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-20)]));
    operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));

    for exp in &profile.experience {
        let line = format!("{} at {} ({})", exp.title, exp.company, exp.date_range);
        // Basic sanitization for PDF string literal (lopdf handles escaping mostly, but newlines are tricky)
        let clean_line = line.replace('\n', " ");
        operations.push(Operation::new("Tj", vec![Object::string_literal(clean_line)]));
        operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-15)]));
    }
    
    operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-15)]));

    // Education Header
    operations.push(Operation::new("Tf", vec!["F1".into(), 14.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal("Education")]));
    operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-20)]));
    operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));

    for edu in &profile.education {
        let line = format!("{} - {}", edu.school, edu.degree);
        let clean_line = line.replace('\n', " ");
        operations.push(Operation::new("Tj", vec![Object::string_literal(clean_line)]));
        operations.push(Operation::new("Td", vec![0.into(), Object::Integer(-15)]));
    }

    operations.push(Operation::new("ET", vec![]));

    let content = Content { operations };
    let content_id = doc.add_object(lopdf::Stream::new(dictionary! {}, content.encode().unwrap()));

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources_id,
    });

    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
    };

    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.save(output).map_err(|e| crate::AnalysisError::PdfError(e.to_string()))?;
    Ok(())
}
