//! main entry

use clap::Parser;
use epub::{
    archive::ArchiveError,
    doc::{DocError, EpubDoc},
};
use log::LevelFilter;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Debug logging
    #[arg(long)]
    debug: bool,

    /// Error and warning only logging
    #[arg(long)]
    errors: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// No logging
    #[arg(short, long)]
    silent: bool,

    /// Output directory
    #[arg(short, long)]
    output: Option<String>,

    /// Walk the spine and don't read the TOC
    #[arg(short, long)]
    walk: bool,

    /// EPUB input
    epub_file: String,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    /// Archive errors
    Archive(ArchiveError),

    /// EpubDoc errors
    Doc(DocError),

    /// IO errors
    Io(io::Error),
}

impl From<ArchiveError> for Error {
    fn from(error: ArchiveError) -> Error {
        Error::Archive(error)
    }
}

impl From<DocError> for Error {
    fn from(error: DocError) -> Error {
        Error::Doc(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

fn rename(title: &str) -> String {
    title
        .chars()
        .filter_map(|c| {
            if c == ' ' {
                Some('-')
            } else if c.is_alphanumeric() {
                Some(c)
            } else {
                None
            }
        })
        .collect::<String>()
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    // Initialize logger
    let mut builder = env_logger::Builder::new();
    builder.format_timestamp(None);
    if cli.silent {
        builder.filter_level(LevelFilter::Off);
    } else if cli.errors {
        builder.filter_level(LevelFilter::Warn);
    } else if cli.debug {
        builder.filter_level(LevelFilter::Debug);
    } else if cli.verbose {
        builder
            .filter_level(LevelFilter::Info)
            .filter(Some("epub-dump"), LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }
    builder.init();

    // Open doc
    log::info!("Opening `{}`", cli.epub_file);
    let mut doc = EpubDoc::new(cli.epub_file)?;

    // Make output directory
    let output = match &cli.output {
        Some(odir) => odir,
        None => "output",
    };
    log::info!("Outputting to `{}`", output);
    if !fs::exists(output)? {
        log::info!("Directory `{}` does not exist. Creating.", output);
        fs::create_dir(output)?;
    }

    if !cli.walk && doc.toc.len() > 0 {
        log::info!("Found a table of contents");

        // Read toc
        let mut sections = Vec::new();
        let mut count = 0;
        for nav_point in doc.toc.iter() {
            log::info!("Found {}", nav_point.label);
            count = count + 1;
            let extension = match nav_point.content.extension() {
                Some(ext) => ext.to_str().unwrap_or(""),
                None => "",
            };
            let filename = format!("{:04}-{}.{}", count, &rename(&nav_point.label), extension);
            if let Some(content) = nav_point.content.to_str() {
                sections.push((filename, content.replace("\\", "/")));
            }
        }

        // Write sections
        for (filename, resource) in sections {
            match &doc.get_resource_by_path(&resource) {
                Some(bytes) => {
                    log::info!("Writing content to `{}`", filename);
                    let mut f = fs::File::create(Path::new(output).join(filename))?;
                    f.write_all(bytes)?;
                }
                None => log::error!("Could not open `{}`", resource),
            }
        }
    } else {
        log::warn!("Could not find a table of contents");

        // Walk the book
        let sections = doc
            .spine
            .iter()
            .map(|section| {
                log::info!("Found {}", section.idref);
                section.idref.to_string()
            })
            .collect::<Vec<String>>();

        // Write sections
        for resource in sections {
            match &doc.get_resource(&resource) {
                Some((bytes, _)) => {
                    log::info!("Writing content to `{}`", resource);
                    let mut f = fs::File::create(Path::new(output).join(resource))?;
                    f.write_all(bytes)?;
                }
                None => log::error!("Could not open `{}`", resource),
            }
        }
    }

    Ok(())
}
