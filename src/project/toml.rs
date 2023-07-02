use std::{fs, path::PathBuf};

use crate::{config::ProjectFile, semver::SemanticVersion};
use color_eyre::eyre::{self, WrapErr};
use console::style;
use toml_edit::{Document, Item};
use tracing::{debug, info, instrument, trace, warn};

use super::VersionFile;

#[derive(Debug)]
pub struct Toml {
    toml: Document,
    config: ProjectFile,
}

impl VersionFile for Toml {
    #[instrument(level = "trace", name = "toml::new")]
    fn new(filepath: &PathBuf, config: &ProjectFile) -> eyre::Result<Box<Self>>
    where
        Self: Sized,
    {
        let filecontent = fs::read_to_string(filepath).context("Failed to read project file")?;
        let toml = filecontent
            .parse::<Document>()
            .context("Failed to parse toml file")?;
        let config = config.clone();
        debug!("toml: {:?}", toml);
        Ok(Box::new(Toml {
            toml,
            config,
        }))
    }

    #[instrument(level = "trace", name = "toml::read_version", skip(self))]
    fn read_version(&self) -> eyre::Result<String> {
        // TODO: Figure out how to do that dynamic

        let path: Vec<&str> = self.config.key.split('.').collect();
        let path = path.as_slice();
        let version = get_version(self.toml.as_item(), path, 0)?;

        // let version = self.toml["package"]["version"].as_str().unwrap();
        // let version = version.to_string();
        Ok(version)
    }

    #[instrument(level = "trace", name = "toml::update_project", skip(self))]
    fn update_project(&mut self, semver: &SemanticVersion) -> eyre::Result<String> {
        info!("Updating toml!");
        let path: Vec<&str> = self.config.key.split('.').collect();
        let path = path.as_slice();
        update_version(self.toml.as_item_mut(), path, 0, &semver.to_string())?;
        Ok(self.toml.to_string())
    }
}

#[instrument(level = "trace", name = "toml::get_version", skip(node))]
fn get_version(node: &Item, desired_path: &[&str], current_index: usize) -> eyre::Result<String> {
    trace!("get_version called");
    // At the target key, this "overshoots" on purpose to step down to the actual target node, don't correct this
    if current_index == desired_path.len() {
        trace!("Found target: {:?}", node);
        let val = node.as_str().unwrap();
        let val = val.to_owned();
        return Ok(val);
    }
    let desired_key = desired_path.get(current_index).unwrap();
    if node.is_table() {
        trace!(
            "Item is a table, trying to get key {}",
            style(desired_key).bold()
        );
        let node = node.as_table().unwrap();
        match node.get(desired_key) {
            Some(node) => {
                return get_version(node, desired_path, current_index + 1);
            }
            None => return Err(eyre::eyre!("Failed to find the desired path in the toml")),
        }
    }
    if node.is_array() {
        trace!(
            "Item is an array, trying to get key {}",
            style(desired_key).bold()
        );
        let node = node.as_array().unwrap();
        let desired_key = usize::from_str_radix(desired_key, 10).with_context(|| format!("Error when trying to traverse the toml. Got an array, but {} is not a valid array index.", style(desired_key).bold()))?;
        match node.get(desired_key) {
            Some(node) => {
                let node = Item::Value(node.clone());
                return get_version(&node, desired_path, current_index + 1);
            }
            None => return Err(eyre::eyre!("Failed to find the desired path in the toml")),
        }
    }
    Err(eyre::eyre!("Failed to find the desired key"))
}

#[instrument(level = "trace", name = "toml::update_version", skip(node))]
fn update_version(
    node: &mut Item,
    desired_path: &[&str],
    current_index: usize,
    version: &str,
) -> eyre::Result<()> {
    trace!("get_version called");
    // At the target key, this "overshoots" on purpose to step down to the actual target node, don't correct this
    if current_index == desired_path.len() {
        trace!("Found target: {:?}", node);
        *node = toml_edit::value(version);
        return Ok(());
    }
    let desired_key = desired_path.get(current_index).unwrap();
    if node.is_table() || node.is_array() {
        trace!(
            "Item is a table or array, trying to get key {}",
            style(desired_key).bold()
        );
        let key_as_usize = usize::from_str_radix(desired_key, 10);
        match key_as_usize {
            Ok(desired_key) => match node.get_mut(desired_key) {
                Some(node) => {
                    return update_version(node, desired_path, current_index + 1, version);
                }
                None => return Err(eyre::eyre!("Failed to find the desired path in the toml")),
            },
            Err(_) => match node.get_mut(desired_key) {
                Some(node) => {
                    return update_version(node, desired_path, current_index + 1, version);
                }
                None => return Err(eyre::eyre!("Failed to find the desired path in the toml")),
            },
        }
    }
    Err(eyre::eyre!("Failed to find the desired key"))
}
