//! main entry

use clap::{Parser, Subcommand};
use log::LevelFilter;
use wnrake::error::{Error, ErrorType};

mod build;
mod config;
mod crawl;
mod download;
mod info;
mod parse;
mod utils;

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Debug logging
    #[arg(long)]
    debug: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file
    #[arg(short = 'f')]
    config: Option<String>,

    /// Solver URL [default: http://localhost:8191/v1]
    #[arg(long, value_name = "URL")]
    solver: Option<String>,

    /// Disable cache
    #[arg(long)]
    disable_cache: bool,

    /// Cache [default: disabled]
    #[arg(long, value_name = "DIR")]
    cache: Option<String>,

    /// Name of proxy (in configuration file)
    #[arg(long, value_name = "NAME")]
    proxy: Option<String>,

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

fn load_configuration(
    config: Option<String>,
    solver: Option<String>,
    disable_cache: bool,
    cache: Option<String>,
    proxy_name: Option<String>,
) -> config::Config {
    let config_file = if config.is_some() {
        Some(config.unwrap())
    } else if cfg!(windows) {
        match std::env::var("LOCALAPPDATA") {
            Ok(home) => Some(format!("{}/wnrake.toml", home)),
            _ => None,
        }
    } else {
        match std::env::var("HOME") {
            Ok(home) => Some(format!("{}/.wnrake", home)),
            _ => None,
        }
    };
    log::debug!("config file: {:?}", config_file);
    let config = match &config_file {
        Some(f) => config::ConfigFile::load(f),
        None => config::ConfigFile::default(),
    };
    config::Config::merge(
        config_file,
        solver,
        disable_cache,
        cache,
        proxy_name,
        config,
    )
}

#[tokio::main]
async fn dispatcher() -> Result<(), Error> {
    let cli = Cli::parse();

    // Initialize logger
    let mut builder = env_logger::Builder::new();
    builder.format_timestamp(None);
    if cli.debug {
        builder.filter_level(LevelFilter::Debug);
    } else if cli.verbose {
        builder
            .filter_level(LevelFilter::Info)
            .filter(Some("wnrake"), LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }
    builder.init();

    // Load configuration
    let command = cli.command;
    let config = load_configuration(
        cli.config,
        cli.solver,
        cli.disable_cache,
        cli.cache,
        cli.proxy,
    );
    log::debug!("{:?}", config);

    // Dispatch
    match &command {
        Command::Info(cmd) => cmd.execute(&config).await,
        Command::Download(cmd) => cmd.execute(&config).await,
        Command::Crawl(cmd) => cmd.execute(&config).await,
        Command::Parse(cmd) => cmd.execute(&config).await,
        Command::Build(cmd) => cmd.execute(&config).await,
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
