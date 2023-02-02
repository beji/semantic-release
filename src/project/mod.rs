use anyhow::Context;
use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use tracing::{debug, trace};

use crate::semver::SemanticVersion;

#[derive(Debug, PartialEq, Clone)]
pub enum ProjectType {
    Cargo,
    NodeJs,
    Maven,
    DefaultNix,
}

#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub project_type: ProjectType,
    // TODO: This being a PathBuf prevents Copy from working... Not so nice
    pub filename: PathBuf,
}

#[derive(Debug)]
pub struct RegexResult {
    pub line: usize,
    pub version: String,
}

pub fn find_project_files(path: &str) -> anyhow::Result<Vec<ProjectFile>> {
    let path = Path::new(path);
    let entries = fs::read_dir(path)
        .with_context(|| format!("Failed to read project dir {}", path.to_str().unwrap()))?;

    let mut project_files: Vec<ProjectFile> = vec![];

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let filename = entry.file_name().to_owned();
        let filename = filename
            .to_str()
            .context("Failed to get the filename from a a directory entry")?;

        if filename.ends_with("Cargo.toml") {
            debug!("Found a cargo project file at {}", filename);
            project_files.push(ProjectFile {
                project_type: ProjectType::Cargo,
                filename: entry.path(),
            });
        } else if filename.ends_with("package.json") {
            debug!("Found a node project file at {}", filename);
            project_files.push(ProjectFile {
                project_type: ProjectType::NodeJs,
                filename: entry.path(),
            });
        } else if filename.ends_with("pom.xml") {
            debug!("Found a maven project file at {}", filename);
            project_files.push(ProjectFile {
                project_type: ProjectType::Maven,
                filename: entry.path(),
            });
        } else if filename.ends_with("default.nix") {
            debug!("Found a nix project file at {}", filename);
            project_files.push(ProjectFile {
                project_type: ProjectType::DefaultNix,
                filename: entry.path(),
            });
        }
    }

    Ok(project_files)
}

pub fn read_project_version(project_file: ProjectFile) -> anyhow::Result<Option<RegexResult>> {
    // NOTE: These unwraps should be fine, this is a static string...
    match project_file.project_type {
        ProjectType::Cargo | ProjectType::DefaultNix => {
            let re = Regex::new(r"version\s*=\s*.([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}]).*").unwrap();
            read_project_version_regex(project_file, re)
        }
        ProjectType::NodeJs => {
            let re =
                Regex::new(r"\s*.version.\s*:\s*.([0-9]{1,}\.[0-9]{1,}\.[0-9]{1,}).*").unwrap();
            read_project_version_regex(project_file, re)
        }
        ProjectType::Maven => {
            let re = Regex::new(r"<version>([0-9]{1,}\.[0-9]{1,}\.[0-0{1,}])</version>").unwrap();
            read_project_version_regex(project_file, re)
        }
    }
}

fn read_project_version_regex(
    project_file: ProjectFile,
    re: Regex,
) -> anyhow::Result<Option<RegexResult>> {
    let filename = project_file.filename;
    trace!(
        "Opening {} in search of a project version",
        filename.display()
    );
    let file = File::open(&filename).with_context(|| {
        format!(
            "Failed to open file {}, maybe the user is lacking read permissions?",
            filename.display()
        )
    })?;
    let reader = BufReader::new(file);
    trace!("Opened file");

    // Now search for the first matching line
    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read a line from the file {}, maybe the user is lacking read permissions?", filename.display()))?;
        match re.captures(&line) {
            Some(cap) => {
                // 0 is the whole line, 1 is the capture group we care about
                let cap = &cap[1];
                debug!("Found a matching substring '{}' at line {}", cap, index);

                return Ok(Some(RegexResult {
                    line: index,
                    version: cap.to_owned(),
                }));
            }
            None => (),
        }
    }

    Ok(None)
}

pub fn update_project_version_file(
    file: &ProjectFile,
    version: SemanticVersion,
    line_to_replace: usize,
) -> anyhow::Result<()> {
    let filename = &file.filename;
    debug!("Starting to write {}", filename.display());
    // Need to open as mut so permissions can be restored later on
    let mut project_file = File::open(filename)
        .with_context(|| format!("Failed to open project file {}", filename.display()))?;

    let permissions = project_file
        .metadata()
        .expect("Failed to read project file metadata")
        .permissions();
    let reader = BufReader::new(&mut project_file);
    let mut lines: Vec<String> = vec![];

    for (index, line) in reader.lines().enumerate() {
        if index == line_to_replace {
            lines.push(build_version_line(&file, &version))
        } else {
            let line = line.context("Failed to read line")?;
            lines.push(line);
        }
    }
    debug!("Creating temporary file to write to");
    // Can't use the basic tempfile::tempfile() as that doesn't expose the file path
    // We need that to use fs::copy, see below
    let mut tmp = NamedTempFile::new().context("Failed to create a temporary file")?;
    trace!("Created a temporary file at {}", &tmp.path().display());
    let new_content = lines.join("\n");

    let written_bytes = tmp.write(new_content.as_bytes()).with_context(|| {
        format!(
            "Failed to write to temporary file {}",
            &tmp.path().display()
        )
    })?;
    trace!("Wrote {} bytes", written_bytes);

    debug!(
        "Copying from {} to {}",
        &tmp.path().display(),
        filename.display(),
    );
    let _ = fs::copy(tmp.path(), Path::new(filename)).with_context(|| {
        format!(
            "Failed to copy from {} to {}",
            &tmp.path().display(),
            filename.display()
        )
    })?;
    let _ = project_file
        .set_permissions(permissions)
        .context("Failed to restore file permissions")?;

    Ok(())
}

fn build_version_line(file: &ProjectFile, version: &SemanticVersion) -> String {
    match file.project_type {
        ProjectType::Cargo => format!("version = \"{}\"", version),
        ProjectType::NodeJs => format!("  \"version\": \"{}\",", version),
        ProjectType::Maven => format!("  <version>{}</version>", version),
        ProjectType::DefaultNix => format!("  version = \"{}\";", version),
    }
}
