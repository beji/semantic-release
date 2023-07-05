use std::{fs, path::Path};

use color_eyre::{
    eyre::{self, WrapErr},
    Help,
};
use console::style;
use serde::Deserialize;
use toml_edit::Document;
use tracing::{debug, info};

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum ProjectType {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "toml")]
    Toml,
}

#[derive(Deserialize, Debug)]
pub struct ProjectFile {
    pub path: String,
    pub key: String,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
}

impl Clone for ProjectFile {
    fn clone(&self) -> Self {
        ProjectFile {
            path: self.path.to_string(),
            key: self.key.to_string(),
            project_type: self.project_type,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub tagprefix: String,
    pub subpath: String,
    pub files: Vec<ProjectFile>,
}

impl Config {
    pub fn from_path(path: &str) -> eyre::Result<Config> {
        let path = Path::new(path);
        let path = fs::canonicalize(path)
            .wrap_err_with(|| format!("Failed to turn {} into a valid path", &path.display()))
            .suggestion("If the file doesn't exist you can create it with the --init flag")?;
        info!("Parsing config file {:?}", style(&path).bold());
        let file = fs::read_to_string(&path)
            .wrap_err_with(|| format!("failed to read config file {:?}", &path))
            .suggestion("If the file doesn't exist you can create it with the --init flag")?;

        let config = file.parse::<Document>().unwrap();
        let config: Config = toml_edit::de::from_document(config)
            .wrap_err_with(|| format!("Failed to parse config file {:?}", &path))?;

        debug!("Parsed config: {:?}", config);
        Ok(config)
    }
}
