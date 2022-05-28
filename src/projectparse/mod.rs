use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

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
                break;
            } else if filename.ends_with("package.json") {
                project_type = ProjectType::NodeJs;
                project_file = entry.path();
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

    fn read_project_version_node(&mut self) -> bool {
        lazy_static! {
            // TODO: Figure out how to match " here
            static ref NODE_RE: Regex = Regex::new(r"\s*.version.\s*:\s*.([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}]).*").unwrap();
        }

        let filename = &self.project_file;
        let file = File::open(filename).expect("Failed to open project file");
        let reader = BufReader::new(file);

        let mut has_match = false;

        for (index, line) in reader.lines().enumerate() {
            let line = line.expect("Failed to get a single line from the project file");
            match NODE_RE.captures(&line) {
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

    pub fn read_project_version(&mut self) -> bool {
        match self.project_type {
            ProjectType::NodeJs => self.read_project_version_node(),
            _ => panic!("No idea how to read project type"),
        }
    }
}
