use clap::{arg, command, Command};

use self::logger::LogLevel;

pub mod logger;

pub struct CliContext {
    pub path: String,
    pub tag_prefix: String,
    pub log_level: LogLevel,
    pub dryrun: bool,
}

impl CliContext {
    pub fn new() -> Result<CliContext, &'static str> {
        let matches = command!()
            .arg(arg!([PATH] "Path to the subproject to release"))
            .arg(arg!(-t --tag <TAGPREFIX> "Prefix of the tags to be matched"))
            .arg(arg!(-d --dry ... "Dry run (don't actually change files or do git commits/tags)"))
            .arg(arg!(-v --verbose ... "Log debug informations"))
            .get_matches();

        let log_level = match matches.occurrences_of("verbose") {
            0 => LogLevel::INFO,
            _ => LogLevel::DEBUG,
        };

        let dryrun = match matches.occurrences_of("dry") {
            0 => false,
            _ => true,
        };

        match matches.value_of("PATH") {
            Some(path) => match matches.value_of("tag") {
                Some(tag_prefix) => Ok(CliContext {
                    path: path.to_owned(),
                    tag_prefix: tag_prefix.to_owned(),
                    log_level,
                    dryrun,
                }),
                None => Err("Missing required parameter: tag"),
            },
            None => Err("Missing required parameter: path"),
        }
    }
}
