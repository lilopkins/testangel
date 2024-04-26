use std::fs;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::Path;

use base64::Engine;
use genpdf::style::{Style, StyledString};
use genpdf::{elements, Element};
use testangel_ipc::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportGenerationError {
    #[error("Invalid image format: {0}")]
    InvalidImageFormat(std::io::Error),
    #[error("Invalid image data: {0}")]
    InvalidImageData(image::ImageError),
    #[error("Invalid encoded image data from engine.")]
    InvalidImageBase64Data,
    #[error("Failed to generate image data: {0}")]
    FailedToGenerateImage(image::ImageError),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to generate PDF: {0}")]
    PdfGeneration(#[from] genpdf::error::Error),
}

pub fn save_report<P: AsRef<Path>>(
    to: P,
    evidence: Vec<Evidence>,
) -> Result<(), ReportGenerationError> {
    fs::create_dir_all("./.tafonts")?;
    fs::write(
        "./.tafonts/LiberationSans-Bold.ttf",
        include_bytes!("./fonts/LiberationSans-Bold.ttf"),
    )?;
    fs::write(
        "./.tafonts/LiberationSans-BoldItalic.ttf",
        include_bytes!("./fonts/LiberationSans-BoldItalic.ttf"),
    )?;
    fs::write(
        "./.tafonts/LiberationSans-Italic.ttf",
        include_bytes!("./fonts/LiberationSans-Italic.ttf"),
    )?;
    fs::write(
        "./.tafonts/LiberationSans-Regular.ttf",
        include_bytes!("./fonts/LiberationSans-Regular.ttf"),
    )?;

    let font_family = genpdf::fonts::from_files("./.tafonts", "LiberationSans", None)?;
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("TestAngel Evidence");
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    decorator.set_header(|page_no| {
        elements::PaddedElement::new(
            elements::LinearLayout::vertical()
                .element(elements::Text::new(StyledString::new(
                    "Flow Evidence",
                    Style::new().bold().with_font_size(18),
                )))
                .element(elements::Text::new(StyledString::new(
                    format!(
                        "Page {page_no} - Generated by TestAngel at {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M")
                    ),
                    Style::new().with_font_size(10),
                ))),
            (0, 0, 4, 0),
        )
    });
    doc.set_page_decorator(decorator);

    for ev in &evidence {
        doc.push(elements::Paragraph::new(ev.label.clone()).padded((3, 0, 0, 0)));
        match &ev.content {
            EvidenceContent::Textual(text) => {
                for para in text.split('\n') {
                    doc.push(elements::Paragraph::new(para));
                }
            }
            EvidenceContent::ImageAsPngBase64(base64) => {
                let data = base64::engine::general_purpose::STANDARD
                    .decode(base64)
                    .map_err(|_| ReportGenerationError::InvalidImageBase64Data)?;

                // Make sure it's encoded as expected, fixing #107
                let img = image::io::Reader::new(BufReader::new(Cursor::new(data)))
                    .with_guessed_format()
                    .map_err(ReportGenerationError::InvalidImageFormat)?
                    .decode()
                    .map_err(ReportGenerationError::InvalidImageData)?
                    .into_rgb8();
                let mut data = vec![];
                img.write_to(
                    &mut BufWriter::new(Cursor::new(&mut data)),
                    image::ImageFormat::Png,
                )
                .map_err(ReportGenerationError::FailedToGenerateImage)?;

                doc.push(elements::Image::from_reader(Cursor::new(data))?);
            }
        }
    }

    doc.render_to_file(to.as_ref().with_extension("pdf"))?;
    fs::remove_dir_all("./.tafonts")?;

    Ok(())
}
