//! main entry

use clap::{Parser, Subcommand};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use wnrake::error::{Error, ErrorType};

mod build;
mod config;
mod crawl;
mod download;
mod info;
mod parse;

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Verbose logging
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Configuration file
    #[arg(short = 'f')]
    config: Option<String>,

    /// Client parameters
    #[command(flatten)]
    config_params: config::ConfigParams,

    /// Command
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
#[clap(disable_help_subcommand = true)]
enum Command {
    /// Fetches book information
    Info(info::Info),

    /// Downloads URL cache
    Download(download::Download),

    /// Crawls through novel chapters
    Crawl(crawl::Crawl),

    /// Parses downloaded chapters
    Parse(parse::Parse),

    /// Builds epub book
    Build(build::Build),
}

fn load_configuration(cli: &mut Cli) {
    let config_file = if cli.config.is_some() {
        Some(cli.config.clone().unwrap())
    } else if cfg!(windows) {
        match std::env::var("APP_DATA") {
            Ok(app_data) => Some(format!("{}/wnrake.toml", app_data)),
            _ => None,
        }
    } else {
        match std::env::var("HOME") {
            Ok(home) => Some(format!("{}/.wnrake", home)),
            _ => None,
        }
    };
    if let Some(config_file) = config_file {
        cli.config_params.load_config(config_file.as_ref());
    }
}

#[tokio::main]
async fn dispatcher() -> Result<(), Error> {
    let mut cli = Cli::parse();

    // Initialize
    SimpleLogger::new()
        .with_colors(true)
        .with_level(match cli.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init()
        .expect("failed to initialize logger");
    load_configuration(&mut cli);

    // Dispatch
    match &cli.command {
        Command::Info(cmd) => cmd.execute(&cli.config_params).await,
        Command::Download(cmd) => cmd.execute(&cli.config_params).await,
        Command::Crawl(cmd) => cmd.execute(&cli.config_params).await,
        Command::Parse(cmd) => cmd.execute(&cli.config_params).await,
        Command::Build(cmd) => cmd.execute(&cli.config_params).await,
    }
}

fn main() {
    std::process::exit(match dispatcher() {
        Err(e) => {
            log::error!("{}", e);
            match e.error_type {
                ErrorType::Html => 1,
                ErrorType::Io => 2,
                ErrorType::Json => 3,
                ErrorType::Parser => 4,
                ErrorType::Solution => 5,
                ErrorType::Solver => 6,
                ErrorType::Status => 7,
                ErrorType::Timeout => 8,
            }
        }
        Ok(_) => 0,
    })
}
