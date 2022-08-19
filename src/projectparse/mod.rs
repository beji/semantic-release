use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

use crate::{cli::logger::Logger, semver::SemanticVersion};

use self::{cargo::CargoProject, node::NodeProject, pom::PomProject};

mod cargo;
mod node;
mod pom;

#[derive(PartialEq)]
pub enum ProjectType {
    Cargo,
    NodeJs,
    PomXml,
    Unknown,
}

pub trait Project {
    fn read_project_version_regex(&mut self, re: Regex) -> bool {
        let filename = &self.get_project_file();
        let logger = &self.get_logger();
        let file = File::open(filename).expect("Failed to open project file");
        let reader = BufReader::new(file);

        let mut has_match = false;

        for (index, line) in reader.lines().enumerate() {
            let line = line.expect("Failed to get a single line from the project file");
            match re.captures(&line) {
                Some(cap) => {
                    // 0 is the whole line, 1 is the capture group we care about
                    let cap = &cap[1];
                    logger.log_debug(format!("Matched '{}' at line {}", cap, index));

                    self.set_found_line(index);
                    self.set_version_string(cap.to_owned());
                    has_match = true;
                    break;
                }
                None => (),
            }
        }
        has_match
    }

    fn update_project_version_file(&self, next_version: &SemanticVersion) {
        let filename = self.get_project_file();
        let mut project_file = File::open(filename).expect("Failed to open project file");
        // Need to preserve correct permissions (maybe, not sure actually)
        let permissions = project_file
            .metadata()
            .expect("Failed to read project file metadata")
            .permissions();
        let reader = BufReader::new(&mut project_file);
        let mut lines: Vec<String> = vec![];

        let new_version_line = self.build_project_line(next_version);

        for (index, line) in reader.lines().enumerate() {
            if index == self.get_found_line() {
                lines.push(new_version_line.clone());
            } else {
                let line = line.expect("Failed to read line");
                lines.push(line);
            }
        }

        // Push an empty string so the join creates a newline at the file end
        lines.push("".to_string());

        // Can't use the basic tempfile::tempfile() as that doesn't expose the file path
        // We need that to use fs::copy, see below
        let mut tmpfile = NamedTempFile::new().expect("Failed to create temporary file");
        let new_content = lines.join("\n");

        let written = tmpfile
            .write(new_content.as_bytes())
            .expect("Failed to write to temporary file");

        self.get_logger()
            .log_debug(format!("written bytes: {}", written));

        // io::copy lead to bad file descriptor issues (maybe one of the file handles is closed before the copy happens?)
        fs::copy(tmpfile.path(), Path::new(filename)).expect("failed to copy");
        project_file
            .set_permissions(permissions)
            .expect("Failed to set file permissions");
    }

    fn read_project_version(&mut self) -> bool;
    fn get_project_file(&self) -> &PathBuf;
    fn get_logger(&self) -> &Logger;
    fn set_found_line(&mut self, line: usize);
    fn get_found_line(&self) -> usize;
    fn get_version_string(&self) -> &str;
    fn set_version_string(&mut self, version_string: String);
    fn build_project_line(&self, next_version: &SemanticVersion) -> String;
    fn get_project_type(&self) -> ProjectType;
}

pub fn new_project<'a>(path: &'a str, logger: &'a Logger) -> Box<dyn Project + 'a> {
    let path = Path::new(path);
    let entries = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read project dir {}", path.to_str().unwrap()));

    let mut project_type = ProjectType::Unknown;
    let mut project_file = PathBuf::new();

    for entry in entries {
        let entry = entry.expect("Failed to get entry");

        let filename = entry
            .file_name()
            .into_string()
            .expect("Failed to parse file name to string");
        if filename.ends_with("Cargo.toml") {
            project_type = ProjectType::Cargo;
            project_file = entry.path();
            logger.log_debug("Found a cargo project".to_string());
            break;
        } else if filename.ends_with("package.json") {
            project_type = ProjectType::NodeJs;
            project_file = entry.path();
            logger.log_debug("Found a node project".to_string());
            break;
        } else if filename.ends_with("pom.xml") {
            project_type = ProjectType::PomXml;
            project_file = entry.path();
            logger.log_debug("Found a maven project".to_string());
            break;
        }
    }

    match project_type {
        ProjectType::Cargo => Box::new(CargoProject::new(project_file, logger)),
        ProjectType::NodeJs => Box::new(NodeProject::new(project_file, logger)),
        ProjectType::PomXml => Box::new(PomProject::new(project_file, logger)),
        _ => panic!("No idea how to read project type"),
    }
}
