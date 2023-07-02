pub mod toml;

use std::{collections::HashMap, fs, path::PathBuf};

use color_eyre::eyre::{self, WrapErr};
use console::style;
use serde_json::Value;
use tracing::{debug, info, instrument, warn};

use crate::{
    config::{ProjectFile, ProjectType},
    semver::SemanticVersion,
};

use self::toml::Toml;

pub trait VersionFile {
    fn new(filepath: &PathBuf, config: &ProjectFile) -> eyre::Result<Box<Self>>
    where
        Self: Sized;
    fn read_version(&self) -> eyre::Result<String>;
    fn update_project(&mut self, semver: &SemanticVersion) -> eyre::Result<String>;
}

pub fn load_versionfile(
    filepath: &PathBuf,
    config: &ProjectFile,
) -> eyre::Result<Box<dyn VersionFile>> {
    match config.project_type {
        ProjectType::Json => Ok(Json::new(filepath, config)?),
        ProjectType::Toml => Ok(Toml::new(filepath, config)?),
    }
}

#[derive(Debug)]
pub struct Json {
    json: HashMap<String, serde_json::Value>,
    config: ProjectFile,
}

impl VersionFile for Json {
    #[instrument(level = "trace", name = "json::new")]
    fn new(filepath: &PathBuf, config: &ProjectFile) -> eyre::Result<Box<Self>> {
        let filecontent = fs::read_to_string(filepath).context("Failed to read project file")?;
        let json: HashMap<String, serde_json::Value> = serde_json::from_str(&filecontent)?;
        debug!("json: {:?}", json);
        let config = config.clone();
        Ok(Box::new(Json { json, config }))
    }

    #[instrument(level = "trace", name = "json::read_version")]
    fn read_version(&self) -> eyre::Result<String> {
        let version = self.json[&self.config.key].as_str().unwrap();
        let version = version.to_string();
        Ok(version)
    }

    #[instrument(level = "trace", name = "json::update_project")]
    fn update_project(&mut self, semver: &SemanticVersion) -> eyre::Result<String> {
        info!("Updating JSON");
        let key = format!("{}", self.config.key);
        debug!(
            "Trying to insert {} into key {}",
            style(semver.to_string()).bold(),
            style(&key).bold()
        );
        let _ = self
            .json
            .insert(key, Value::String(semver.to_string()))
            .unwrap();
        let json = serde_json::to_string_pretty(&self.json)
            .context("Failed to turn the parsed object back into JSON")?;
        debug!("new json: {}", json);
        Ok(json)
    }
}
