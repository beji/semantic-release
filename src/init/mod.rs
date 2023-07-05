use std::{fs::File, path::Path};

use color_eyre::{
    eyre::{self, WrapErr},
    Help,
};
use console::style;
use std::io::prelude::*;
use tracing::{debug, info, instrument, warn};

use crate::cli::CliContext;

const TEMPLATE: &str = include_str!("project.toml");

#[instrument(level = "trace")]
pub fn init_project(cli_context: &CliContext) -> eyre::Result<()> {
    info!("{} mode called", style("init").bold());
    let path = &cli_context.path;
    let path = Path::new(&path);
    info!(
        "Will create a new config file at {}",
        style(path.display()).bold()
    );
    if path.exists() {
        warn!(
            "A file already exists at {}, not doing anything",
            style(path.display()).bold()
        );
    } else {
        if cli_context.dryrun {
            info!("Dry run is active, not creating a config file");
            debug!(
                "Would create a config file at {}",
                style(path.display()).bold()
            );
        } else {
            let mut file = File::create(path)
                .wrap_err_with(|| format!("Failed to create file at {}", path.display()))
                .suggestion("Check if the location is actually writeable by the user")?;
            file.write_all(TEMPLATE.as_bytes())
                .wrap_err_with(|| format!("Failed to write to file at {}", path.display()))?;
            info!("Created a config file at {}", style(path.display()).bold());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use toml_edit::{de::from_document, Document};

    use crate::config::Config;

    use super::TEMPLATE;

    #[test]
    fn template_should_be_parseable() {
        let config = TEMPLATE.parse::<Document>().unwrap();
        _ = from_document::<Config>(config).unwrap();
    }
}
