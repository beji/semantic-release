use anyhow::bail;
use clap::{arg, command};

pub struct CliContext {
    pub path: String,
    pub tag_prefix: String,
    pub dryrun: bool,
    pub patchtokens: Vec<String>,
    pub minortokens: Vec<String>,
}

static DEFAULT_PATCH_TOKENS: &str = "fix";
static DEFAULT_MINOR_TOKENS: &str = "feat,feature";

impl CliContext {
    pub fn new() -> anyhow::Result<CliContext> {
        let matches = command!()
            .arg(arg!([PATH] "Path to the subproject to release"))
            .arg(arg!(-t --tag [TAGPREFIX] "Prefix of the tags to be matched").default_value(""))
            .arg(arg!(-d --dry ... "Dry run (don't actually change files or do git commits/tags)"))
            .arg(arg!(--patchtokens [PATCHTOKENS] "Tokens that trigger a patch level bump; comma separated list").default_value(DEFAULT_PATCH_TOKENS))
            .arg(arg!(--minortokens [MINORTOKENS] "Tokens that trigger a patch level bump; comma separated list").default_value(DEFAULT_MINOR_TOKENS))
            .get_matches();

        let dryrun = !matches!(matches.occurrences_of("dry"), 0);

        let patchtokens: Vec<String> = matches
            .value_of("patchtokens")
            .unwrap()
            .split(',')
            .map(|s| s.to_owned())
            .collect();

        let minortokens: Vec<String> = matches
            .value_of("minortokens")
            .unwrap()
            .split(',')
            .map(|s| s.to_owned())
            .collect();

        match matches.value_of("PATH") {
            Some(path) => match matches.value_of("tag") {
                Some(tag_prefix) => Ok(CliContext {
                    path: path.to_owned(),
                    tag_prefix: tag_prefix.to_owned(),
                    dryrun,
                    patchtokens,
                    minortokens,
                }),
                None => bail!("Missing required parameter: tag"),
            },
            None => bail!("Missing required parameter: path"),
        }
    }
}
