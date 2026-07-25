//! main entry

use clap::{Parser, Subcommand};
use crawler::config::{Config, ConfigBuilder};
use log::LevelFilter;

mod debug;
mod error;
mod session;

use error::Error;

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

    /// Cache [default: disabled]
    #[arg(long, value_name = "DIR")]
    cache: Option<String>,

    /// Disable proxy
    #[arg(long)]
    disable_proxy: bool,

    /// Name of proxy (in configuration file)
    #[arg(long, value_name = "NAME")]
    proxy: Option<String>,

    /// Username
    #[arg(short = 'u', long)]
    username: String,

    /// Password
    #[arg(short = 'p', long)]
    password: String,

    /// Command
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
#[clap(disable_help_subcommand = true)]
enum Command {
    /// Helpful for debugging FlareSolverr
    Debug(debug::Debug),
}

fn load_configuration(
    config: Option<String>,
    solver: Option<String>,
    cache: Option<String>,
    disable_proxy: bool,
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
        .disable_cache(true)
        .proxy(proxy_name)
        .disable_proxy(disable_proxy)
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
            .filter(Some("ptrake"), LevelFilter::Debug)
            .filter(Some("crawler"), LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }
    builder.init();

    // Load configuration
    let command = cli.command;
    let config = load_configuration(
        cli.config,
        cli.solver,
        cli.cache,
        cli.disable_proxy,
        cli.proxy,
    )?;
    log::debug!("{:?}", config);

    // Dispatch
    match &command {
        Command::Debug(cmd) => cmd.execute(&cli.username, &cli.password, &config).await,
    }
}

fn main() {
    std::process::exit(match dispatcher() {
        Err(e) => {
            log::error!("{}", e);
            1
        }
        Ok(_) => 0,
    })
    /*
    std::process::exit(match dispatcher() {
        Err(e) => {
            log::error!("{}", e);
            match e.error_type {
                ErrorType::Config => 1,
                ErrorType::Crawler => 2,
                ErrorType::Epub => 3,
                ErrorType::Html => 4,
                ErrorType::Io => 5,
                ErrorType::Json => 6,
                ErrorType::Parser => 7,
            }
        }
        Ok(_) => 0,
    })
    */
}
