use clap::{arg, command, Command};

use self::logger::LogLevel;

pub mod logger;

pub struct CliContext {
    pub path: String,
    pub tag_prefix: String,
    pub log_level: LogLevel,
    pub dryrun: bool,
    pub patchtokens: Vec<String>,
    pub minortokens: Vec<String>,
}

static DEFAULT_PATCH_TOKENS: &str = "fix";
static DEFAULT_MINOR_TOKENS: &str = "feat,feature";

impl CliContext {
    pub fn new() -> Result<CliContext, &'static str> {
        let matches = command!()
            .arg(arg!([PATH] "Path to the subproject to release"))
            .arg(arg!(-t --tag [TAGPREFIX] "Prefix of the tags to be matched").default_value(""))
            .arg(arg!(-d --dry ... "Dry run (don't actually change files or do git commits/tags)"))
            .arg(arg!(-v --verbose ... "Log debug informations"))
            .arg(arg!(--patchtokens [PATCHTOKENS] "Tokens that trigger a patch level bump; comma separated list").default_value(DEFAULT_PATCH_TOKENS))
            .arg(arg!(--minortokens [MINORTOKENS] "Tokens that trigger a patch level bump; comma separated list").default_value(DEFAULT_MINOR_TOKENS))
            .get_matches();

        let log_level = match matches.occurrences_of("verbose") {
            0 => LogLevel::INFO,
            _ => LogLevel::DEBUG,
        };

        let dryrun = match matches.occurrences_of("dry") {
            0 => false,
            _ => true,
        };

        let patchtokens: Vec<String> = matches
            .value_of("patchtokens")
            .unwrap()
            .split(",")
            .map(|s| s.to_owned())
            .collect();

        let minortokens: Vec<String> = matches
            .value_of("minortokens")
            .unwrap()
            .split(",")
            .map(|s| s.to_owned())
            .collect();

        match matches.value_of("PATH") {
            Some(path) => match matches.value_of("tag") {
                Some(tag_prefix) => Ok(CliContext {
                    path: path.to_owned(),
                    tag_prefix: tag_prefix.to_owned(),
                    log_level,
                    dryrun,
                    patchtokens,
                    minortokens,
                }),
                None => Err("Missing required parameter: tag"),
            },
            None => Err("Missing required parameter: path"),
        }
    }
}
