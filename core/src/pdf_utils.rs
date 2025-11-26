use lopdf::{Document, Object, dictionary, content::{Content, Operation}};
use crate::Result;
use crate::AnalysisError;

/// Adds text to a specific page at given coordinates.
pub fn add_text_to_page(
    doc: &mut Document,
    page_number: u32,
    text: &str,
    x: f64,
    y: f64,
    font_size: f64,
    color_gray: f64,
) -> Result<()> {
    let pages = doc.get_pages();
    let page_id = *pages.get(&page_number).ok_or_else(|| AnalysisError::PdfError(format!("Page {} not found", page_number)))?;

    // Ensure font exists
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    // Add font to resources
    let page = doc.get_object(page_id).unwrap().as_dict().unwrap();
    let resources_id = match page.get(b"Resources") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            // If missing or direct, create a new indirect object for resources
            // Note: If it was direct, we lose it here (simplified for now)
            let res_id = doc.add_object(dictionary! {});
            let page_mut = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
            page_mut.set("Resources", Object::Reference(res_id));
            res_id
        }
    };

    if let Ok(resources) = doc.get_object_mut(resources_id) {
        if let Object::Dictionary(dict) = resources {
            if !dict.has(b"Font") {
                dict.set("Font", dictionary! {});
            }
            let fonts = dict.get_mut(b"Font").unwrap().as_dict_mut().unwrap();
            fonts.set("F1", Object::Reference(font_id));
        }
    }

    // Create content stream
    let mut operations = Vec::new();
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec!["F1".into(), font_size.into()]));
    operations.push(Operation::new("g", vec![color_gray.into()]));
    operations.push(Operation::new("Td", vec![x.into(), y.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal(text)]));
    operations.push(Operation::new("ET", vec![]));

    let content = Content { operations };
    let content_stream = doc.add_object(lopdf::Stream::new(dictionary! {}, content.encode().unwrap()));

    // Append to page contents
    let page = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
    
    // Determine the action based on current "Contents"
    enum Action {
        ReplaceWithArray(Vec<Object>),
        AppendToArray,
        SetNew(Object),
    }

    let action = if let Ok(contents) = page.get(b"Contents") {
        match contents {
            Object::Reference(id) => Action::ReplaceWithArray(vec![Object::Reference(*id), Object::Reference(content_stream)]),
            Object::Array(_) => Action::AppendToArray,
            _ => Action::SetNew(Object::Reference(content_stream)),
        }
    } else {
        Action::SetNew(Object::Reference(content_stream))
    };

    match action {
        Action::ReplaceWithArray(arr) => {
            page.set("Contents", arr);
        }
        Action::AppendToArray => {
            if let Ok(Object::Array(arr)) = page.get_mut(b"Contents") {
                arr.push(Object::Reference(content_stream));
            }
        }
        Action::SetNew(obj) => {
            page.set("Contents", obj);
        }
    }

    Ok(())
}

/// Adds text to a specific page at given coordinates, ensuring it is rendered *before* existing content (underlay).
pub fn prepend_text_to_page(
    doc: &mut Document,
    page_number: u32,
    text: &str,
    x: f64,
    y: f64,
    font_size: f64,
    color_gray: f64,
) -> Result<()> {
    let pages = doc.get_pages();
    let page_id = *pages.get(&page_number).ok_or_else(|| AnalysisError::PdfError(format!("Page {} not found", page_number)))?;

    // Ensure font exists (reuse logic or refactor if needed, for now duplicating for safety)
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    // Add font to resources
    let page = doc.get_object(page_id).unwrap().as_dict().unwrap();
    let resources_id = match page.get(b"Resources") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            let res_id = doc.add_object(dictionary! {});
            let page_mut = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
            page_mut.set("Resources", Object::Reference(res_id));
            res_id
        }
    };

    if let Ok(resources) = doc.get_object_mut(resources_id) {
        if let Object::Dictionary(dict) = resources {
            if !dict.has(b"Font") {
                dict.set("Font", dictionary! {});
            }
            let fonts = dict.get_mut(b"Font").unwrap().as_dict_mut().unwrap();
            fonts.set("F1", Object::Reference(font_id));
        }
    }

    // Create content stream
    let mut operations = Vec::new();
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec!["F1".into(), font_size.into()]));
    operations.push(Operation::new("g", vec![color_gray.into()]));
    operations.push(Operation::new("Td", vec![x.into(), y.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal(text)]));
    operations.push(Operation::new("ET", vec![]));

    let content = Content { operations };
    let content_stream = doc.add_object(lopdf::Stream::new(dictionary! {}, content.encode().unwrap()));

    // Prepend to page contents
    let page = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
    
    enum Action {
        ReplaceWithArray(Vec<Object>),
        PrependToArray,
        SetNew(Object),
    }

    let action = if let Ok(contents) = page.get(b"Contents") {
        match contents {
            Object::Reference(id) => Action::ReplaceWithArray(vec![Object::Reference(content_stream), Object::Reference(*id)]),
            Object::Array(_) => Action::PrependToArray,
            _ => Action::SetNew(Object::Reference(content_stream)),
        }
    } else {
        Action::SetNew(Object::Reference(content_stream))
    };

    match action {
        Action::ReplaceWithArray(arr) => {
            page.set("Contents", arr);
        }
        Action::PrependToArray => {
            if let Ok(Object::Array(arr)) = page.get_mut(b"Contents") {
                arr.insert(0, Object::Reference(content_stream));
            }
        }
        Action::SetNew(obj) => {
            page.set("Contents", obj);
        }
    }

    Ok(())
}

/// Creates a blank PDF document.
pub fn create_blank_pdf() -> Document {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
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
    doc
}
