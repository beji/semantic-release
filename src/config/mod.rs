use std::{fs, path::Path};

use color_eyre::eyre::{self, WrapErr};
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
            path: format!("{}", self.path),
            key: format!("{}", self.key),
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
        let path = fs::canonicalize(path)?;
        info!("Parsing config file {:?}", style(&path).bold());
        let file = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file {:?}", &path))?;

        let config = file.parse::<Document>().unwrap();
        let config: Config = toml_edit::de::from_document(config)
            .with_context(|| format!("Failed to parse config file {:?}", &path))?;

        debug!("Parsed config: {:?}", config);
        Ok(config)
    }
}
