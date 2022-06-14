use std::path::PathBuf;

use regex::Regex;

use crate::{cli::logger::Logger, semver::SemanticVersion};

use super::{Project, ProjectType};

pub struct PomProject<'a> {
    project_file: PathBuf,
    found_line: usize,
    version_string: String,
    logger: &'a Logger,
}

impl PomProject<'_> {
    pub fn new<'a>(project_file: PathBuf, logger: &'a Logger) -> PomProject {
        PomProject {
            project_file,
            found_line: usize::MAX,
            version_string: "".to_string(),
            logger,
        }
    }
}

impl Project for PomProject<'_> {
    fn read_project_version(&mut self) -> bool {
        self.read_project_version_regex(
            Regex::new(r"<version>([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}])</version>").unwrap(),
        )
    }
    fn get_project_file(&self) -> &PathBuf {
        &self.project_file
    }

    fn get_logger(&self) -> &Logger {
        self.logger
    }

    fn set_found_line(&mut self, line: usize) -> () {
        self.found_line = line;
    }

    fn set_version_string(&mut self, version_string: String) -> () {
        self.version_string = version_string;
    }

    fn get_found_line(&self) -> usize {
        self.found_line
    }

    fn build_project_line(&self, next_version: &SemanticVersion) -> String {
        format!("  <version>{}</version>", next_version.to_string())
    }

    fn get_project_type(&self) -> ProjectType {
        ProjectType::PomXml
    }
    fn get_version_string(&self) -> &str {
        &self.version_string
    }
}
