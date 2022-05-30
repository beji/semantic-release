#[macro_use]
extern crate lazy_static;

use console::style;

use crate::cli::CliContext;
use crate::git::{calc_bumplevel, GitContext};
use crate::projectparse::{Project, ProjectType};
use crate::semver::SemanticVersion;

mod cli;
mod git;
mod projectparse;
mod semver;

fn main() {
    let cli_context = CliContext::new().expect("Failed to build CLI Context");

    cli_context.log_info(format!(
        "Looking for a git repo at (or above) {}",
        style(&cli_context.path).bold()
    ));
    let git_context = GitContext::new(&cli_context);

    let latest = git_context.get_latest_tag(&cli_context.tag_prefix).unwrap();
    cli_context.log_info(format!(
        "Found tag {}, will use that as base",
        style(&latest.name).bold()
    ));

    let relevant_commits = git_context.get_commits_since_tag(latest);

    if relevant_commits.len() != 0 {
        cli_context.log_debug("Found the following relevant commits:".to_string());
        relevant_commits
            .iter()
            .for_each(|commit| cli_context.log_debug(format!("commit: {:?}", commit)));

        let bumplevel = calc_bumplevel(&relevant_commits);

        let mut project = Project::new(&cli_context.path);

        if project.project_type == ProjectType::Unknown {
            panic!("The project type isn't currently implemented");
        }
        if project.read_project_version() {
            let mut version = SemanticVersion::new(&project.version_string)
                .expect("Failed to parse version string");
            version.bump(bumplevel);

            cli_context.log_debug(format!(
                "bump level: {:?} => next version: {}",
                bumplevel,
                style(&version.to_string()).bold()
            ));
        } else {
            panic!("Failed to find a version string");
        }
    } else {
        cli_context.log_info("Found no commits since the last tag".to_string())
    }
}
