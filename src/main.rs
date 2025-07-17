//! main entry

use clap::{Parser, Subcommand};
use log::LevelFilter;
use wnrake::{
    config::{Config, ConfigBuilder},
    error::{Error, ErrorType},
};

mod build;
mod crawl;
mod debug;
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

    /// Error and warning only logging
    #[arg(long)]
    errors: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// No logging
    #[arg(short, long)]
    silent: bool,

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

    /// Builds epub book
    Debug(debug::Debug),
}

fn load_configuration(
    config: Option<String>,
    solver: Option<String>,
    disable_cache: bool,
    cache: Option<String>,
    proxy_name: Option<String>,
) -> Result<Config, Error> {
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
    let builder = match &config_file {
        Some(f) => ConfigBuilder::new(&f)?,
        None => ConfigBuilder::default(),
    };
    Ok(builder
        .solver(solver)
        .cache(cache)
        .proxy(proxy_name)
        .disable_cache(disable_cache)
        .build())
}

#[tokio::main]
async fn dispatcher() -> Result<(), Error> {
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
    )?;
    log::debug!("{:?}", config);

    // Dispatch
    match &command {
        Command::Info(cmd) => cmd.execute(&config).await,
        Command::Download(cmd) => cmd.execute(&config).await,
        Command::Crawl(cmd) => cmd.execute(&config).await,
        Command::Parse(cmd) => cmd.execute(&config).await,
        Command::Build(cmd) => cmd.execute(&config),
        Command::Debug(cmd) => cmd.execute(&config).await,
    }
}

fn main() {
    std::process::exit(match dispatcher() {
        Err(e) => {
            log::error!("{}", e);
            match e.error_type {
                ErrorType::Config => 1,
                ErrorType::Epub => 2,
                ErrorType::Html => 3,
                ErrorType::Io => 4,
                ErrorType::Json => 5,
                ErrorType::Parser => 6,
                ErrorType::Proxy => 7,
                ErrorType::Solution => 8,
                ErrorType::Solver => 9,
                ErrorType::Status => 10,
            }
        }
        Ok(_) => 0,
    })
}
