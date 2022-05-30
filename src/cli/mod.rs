use clap::{arg, command, Command};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum LogLevel {
    DEBUG,
    INFO,
}

pub struct CliContext {
    pub path: String,
    pub tag_prefix: String,
    log_level: LogLevel,
}

impl CliContext {
    pub fn new() -> Result<CliContext, &'static str> {
        let matches = command!()
            .arg(arg!(-p --path <PATH> "Path to the subproject to release"))
            .arg(arg!(-t --tag <TAGPREFIX> "Prefix of the tags to be matched"))
            .arg(arg!(-v --verbose ... "Log debug informations"))
            .get_matches();

        let log_level = match matches.occurrences_of("verbose") {
            0 => LogLevel::INFO,
            _ => LogLevel::DEBUG,
        };

        match matches.value_of("path") {
            Some(path) => match matches.value_of("tag") {
                Some(tag_prefix) => Ok(CliContext {
                    path: path.to_owned(),
                    tag_prefix: tag_prefix.to_owned(),
                    log_level,
                }),
                None => Err("Missing required parameter: tag"),
            },
            None => Err("Missing required parameter: path"),
        }
    }
    pub fn log_debug(&self, message: String) {
        if self.log_level <= LogLevel::DEBUG {
            eprintln!("[DEBUG]: {}", message);
        }
    }
    pub fn log_info(&self, message: String) {
        if self.log_level <= LogLevel::INFO {
            eprintln!("[INFO]: {}", message);
        }
    }
}
