use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(PartialEq)]
pub enum ProjectType {
    Cargo,
    NodeJs,
    Unknown,
}

pub struct Project {
    pub project_type: ProjectType,
    project_file: PathBuf,
    found_line: usize,
    pub version_string: String,
}

impl Project {
    pub fn new(path: &str) -> Project {
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
                println!("Found a cargo project");
                break;
            } else if filename.ends_with("package.json") {
                project_type = ProjectType::NodeJs;
                project_file = entry.path();
                println!("Found a node project");
                break;
            }
        }

        Project {
            project_type,
            project_file,
            found_line: usize::MAX,
            version_string: "".to_string(),
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
                    println!("Matched '{}' at line {}", cap.to_string(), index);
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
                Regex::new(r"\s*.version.\s*:\s*.([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}]).*").unwrap(),
            ),
            ProjectType::Cargo => self.read_project_version_regex(
                Regex::new(r"version\s*=\s*.([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}]).*").unwrap(),
            ),
            _ => panic!("No idea how to read project type"),
        }
    }
}
