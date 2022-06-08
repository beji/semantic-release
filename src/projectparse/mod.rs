use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

use crate::{cli::logger::Logger, semver::SemanticVersion};

#[derive(PartialEq)]
pub enum ProjectType {
    Cargo,
    NodeJs,
    PomXml,
    Unknown,
}

pub struct Project<'a> {
    pub project_type: ProjectType,
    pub project_file: PathBuf,
    found_line: usize,
    pub version_string: String,
    logger: &'a Logger,
}

impl Project<'_> {
    pub fn new<'a>(path: &'a str, logger: &'a Logger) -> Project<'a> {
        let path = Path::new(path);
        let entries = fs::read_dir(path)
            .expect(format!("Failed to read project dir {}", path.to_str().unwrap()).as_str());

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
            }
        }

        Project {
            project_type,
            project_file,
            found_line: usize::MAX,
            version_string: "".to_string(),
            logger,
        }
    }

    fn read_project_version_regex(&mut self, re: Regex) -> bool {
        let filename = &self.project_file;
        let file = File::open(filename).expect("Failed to open project file");
        let reader = BufReader::new(file);

        let mut has_match = false;

        for (index, line) in reader.lines().enumerate() {
            let line = line.expect("Failed to get a single line from the project file");
            match re.captures(&line) {
                Some(cap) => {
                    // 0 is the whole line, 1 is the capture group we care about
                    let cap = &cap[1];
                    self.logger.log_debug(format!(
                        "Matched '{}' at line {}",
                        cap.to_string(),
                        index
                    ));

                    self.found_line = index;
                    self.version_string = cap.to_owned();
                    has_match = true;
                    break;
                }
                None => (),
            }
        }
        has_match
    }

    // TODO: Find out if this can be done better with traits
    pub fn read_project_version(&mut self) -> bool {
        // TODO: Figure out how to match " in the regexes (. is used for now)
        match self.project_type {
            ProjectType::NodeJs => self.read_project_version_regex(
                Regex::new(r"\s*.version.\s*:\s*.([0-9]{1,}\.[0-9]{1,}\.[0-9]{1,}).*").unwrap(),
            ),
            ProjectType::Cargo => self.read_project_version_regex(
                Regex::new(r"version\s*=\s*.([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}]).*").unwrap(),
            ),
            ProjectType::PomXml => self.read_project_version_regex(
                Regex::new(r"<version>([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}])</version>").unwrap(),
            ),
            _ => panic!("No idea how to read project type"),
        }
    }

    pub fn update_project_version_file(&self, next_version: &SemanticVersion) {
        let filename = &self.project_file;
        let mut project_file = File::open(filename).expect("Failed to open project file");
        // Need to preserve correct permissions (maybe, not sure actually)
        let permissions = project_file
            .metadata()
            .expect("Failed to read project file metadata")
            .permissions();
        let reader = BufReader::new(&mut project_file);
        let mut lines: Vec<String> = vec![];

        let new_version_line = match self.project_type {
            ProjectType::NodeJs => format!("  \"version\": \"{}\",", next_version.to_string()),
            ProjectType::Cargo => format!("version = \"{}\"", next_version.to_string()),
            ProjectType::PomXml => format!("  <version>{}</version>", next_version.to_string()),
            _ => panic!("No idea how to write project type"),
        };

        for (index, line) in reader.lines().enumerate() {
            if index == self.found_line {
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

        self.logger.log_debug(format!("written bytes: {}", written));

        // io::copy lead to bad file descriptor issues (maybe one of the file handles is closed before the copy happens?)
        fs::copy(tmpfile.path(), Path::new(filename)).expect("failed to copy");
        project_file
            .set_permissions(permissions)
            .expect("Failed to set file permissions");
    }
}
