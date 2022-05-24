use clap::{arg, command, Command};

pub struct CliContext {
    pub path: String,
    pub tag_prefix: String,
}

impl CliContext {
    pub fn new() -> Result<CliContext, &'static str> {
        let matches = command!()
            .arg(arg!(-p --path <PATH> "Path to the subproject to release"))
            .arg(arg!(-t --tag <TAGPREFIX> "Prefix of the tags to be matched"))
            .get_matches();

        match matches.value_of("path") {
            Some(path) => match matches.value_of("tag") {
                Some(tag_prefix) => Ok(CliContext {
                    path: path.to_owned(),
                    tag_prefix: tag_prefix.to_owned(),
                }),
                None => Err("Missing required parameter: tag"),
            },
            None => Err("Missing required parameter: path"),
        }
    }
}
