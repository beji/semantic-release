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
    /// Create a config file at the given path instead of doing any semantic releasing
    #[arg(long, default_value_t = false)]
    init: bool,
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

#[derive(Debug)]
pub struct CliContext {
    pub path: String,
    pub log_level: LevelFilter,
    pub dryrun: bool,
    pub init: bool,
}

impl CliContext {
    pub fn new() -> eyre::Result<CliContext> {
        let cli = CliArgs::parse();

        let log_level = cli.log_level();

        let dryrun = cli.dry;
        let init = cli.init;

        Ok(CliContext {
            path: cli.config,
            log_level,
            dryrun,
            init,
        })
    }
}
