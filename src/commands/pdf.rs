use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use clap::Subcommand;
use lopdf::{Document, Object, ObjectId};

#[derive(clap::Args)]
pub struct PdfOpts {
    #[command(subcommand)]
    command: PdfCommand,
}

#[derive(Subcommand)]
enum PdfCommand {
    #[command(about = "查看 PDF 基本信息")]
    Info {
        #[arg(value_name = "input.pdf", help = "输入 PDF 文件")]
        input: PathBuf,
    },
    #[command(about = "按页拆分 PDF")]
    Split {
        #[arg(value_name = "input.pdf", help = "输入 PDF 文件")]
        input: PathBuf,
        #[arg(short, long, value_name = "DIR", help = "输出目录")]
        output_dir: PathBuf,
        #[arg(
            long,
            value_name = "PREFIX",
            default_value = "page",
            help = "输出文件名前缀"
        )]
        prefix: String,
    },
    #[command(about = "合并多个 PDF")]
    Merge {
        #[arg(value_name = "input.pdf", required = true, num_args = 1.., help = "输入 PDF 文件列表")]
        inputs: Vec<PathBuf>,
        #[arg(short, long, value_name = "output.pdf", help = "输出 PDF 文件")]
        output: PathBuf,
    },
}

pub fn run_pdf(opts: PdfOpts) -> Result<(), PdfError> {
    match opts.command {
        PdfCommand::Info { input } => print_info(input),
        PdfCommand::Split {
            input,
            output_dir,
            prefix,
        } => split_pdf(input, output_dir, prefix),
        PdfCommand::Merge { inputs, output } => merge_pdfs(inputs, output),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PdfError {
    #[error("pdf load failed: {0}")]
    Load(String),
    #[error("pdf save failed: {0}")]
    Save(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("file system error: {0}")]
    FileSystem(String),
}

fn print_info(input: PathBuf) -> Result<(), PdfError> {
    let doc = load_document(&input)?;
    let pages = doc.get_pages();

    println!("File: {}", input.display());
    println!("Pages: {}", pages.len());
    println!("Objects: {}", doc.objects.len());
    println!("Version: {}", doc.version);
    if doc.is_encrypted() {
        println!("Encrypted: yes");
    } else {
        println!("Encrypted: no");
    }

    Ok(())
}

fn split_pdf(input: PathBuf, output_dir: PathBuf, prefix: String) -> Result<(), PdfError> {
    if prefix.trim().is_empty() {
        return Err(PdfError::InvalidInput(
            "output prefix cannot be empty".to_string(),
        ));
    }

    fs::create_dir_all(&output_dir)
        .map_err(|e| PdfError::FileSystem(format!("{}: {}", output_dir.display(), e)))?;

    let pages = load_document(&input)?.get_pages();
    if pages.is_empty() {
        return Err(PdfError::InvalidInput("pdf contains no pages".to_string()));
    }

    for page_number in pages.keys() {
        let mut page_doc = load_document(&input)?;
        let remove_pages = pages
            .keys()
            .copied()
            .filter(|number| number != page_number)
            .collect::<Vec<_>>();
        page_doc.delete_pages(&remove_pages);
        page_doc.prune_objects();
        page_doc.renumber_objects();
        page_doc.compress();

        let output = output_dir.join(format!("{}-{}.pdf", prefix, page_number));
        page_doc
            .save(&output)
            .map_err(|e| PdfError::Save(format!("{}: {}", output.display(), e)))?;
        println!("Wrote {}", output.display());
    }

    Ok(())
}

fn merge_pdfs(inputs: Vec<PathBuf>, output: PathBuf) -> Result<(), PdfError> {
    if inputs.len() < 2 {
        return Err(PdfError::InvalidInput(
            "merge requires at least 2 input PDF files".to_string(),
        ));
    }

    let mut merged = merge_documents(inputs)?;

    merged.prune_objects();
    merged.renumber_objects();
    merged.compress();
    merged
        .save(&output)
        .map_err(|e| PdfError::Save(format!("{}: {}", output.display(), e)))?;

    println!("Wrote {}", output.display());
    Ok(())
}

fn merge_documents(inputs: Vec<PathBuf>) -> Result<Document, PdfError> {
    let mut max_id = 1;
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();

    for input in &inputs {
        let mut doc = load_document(input)?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        let pages = doc.get_pages();
        for (_, object_id) in pages {
            let object = doc
                .get_object(object_id)
                .map_err(|e| PdfError::Load(format!("{}: {}", input.display(), e)))?
                .to_owned();
            documents_pages.insert(object_id, object);
        }
        documents_objects.extend(doc.objects);
    }

    let mut document = Document::with_version("1.5");
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                catalog_object = Some((
                    catalog_object.map(|(id, _)| id).unwrap_or(object_id),
                    object,
                ));
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, old_object)) = &pages_object {
                        if let Ok(old_dictionary) = old_object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        pages_object.map(|(id, _)| id).unwrap_or(object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                document.objects.insert(object_id, object);
            }
        }
    }

    let (catalog_id, catalog_object) = catalog_object
        .ok_or_else(|| PdfError::InvalidInput("catalog root not found".to_string()))?;
    let (page_id, page_object) =
        pages_object.ok_or_else(|| PdfError::InvalidInput("pages root not found".to_string()))?;

    for (object_id, object) in &documents_pages {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", page_id);
            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = page_object.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages
                .keys()
                .copied()
                .map(|id| Object::Reference(id))
                .collect::<Vec<_>>(),
        );
        document
            .objects
            .insert(page_id, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", page_id);
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_id);
    document.max_id = document.objects.len() as u32;

    Ok(document)
}

fn load_document(input: &PathBuf) -> Result<Document, PdfError> {
    if !input.is_file() {
        return Err(PdfError::InvalidInput(format!(
            "{} is not a file",
            input.display()
        )));
    }

    Document::load(input).map_err(|e| PdfError::Load(format!("{}: {}", input.display(), e)))
}
