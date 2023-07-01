use clap::{arg, Parser};
use color_eyre::eyre;
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser, Debug)]
struct CliArgs {
    /// Log debug infos, may be passed more than once to increase log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Don't actually change any files or do git commits/tags
    #[arg(short, long, default_value_t = false)]
    dry: bool,
    config: String,
}
impl CliArgs {
    pub fn log_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}

pub struct CliContext {
    pub path: String,
    pub log_level: LevelFilter,
    pub dryrun: bool,
}

impl CliContext {
    pub fn new() -> eyre::Result<CliContext> {
        let cli = CliArgs::parse();

        let log_level = cli.log_level();

        let dryrun = cli.dry;

        Ok(CliContext {
            path: cli.config,
            log_level,
            dryrun,
        })
    }
}
