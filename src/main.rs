use crate::cli::CliContext;
use crate::git::{calc_bumplevel, GitContext};
use crate::projectparse::Project;
use crate::semver::SemanticVersion;

mod cli;
mod git;
mod projectparse;
mod semver;

fn main() {
    let cli_context = CliContext::new().expect("Failed to build CLI Context");

    println!("Looking for a git repo at (or above) {}", cli_context.path);
    let git_context = GitContext::new(&cli_context.path);

    let latest = git_context.get_latest_tag(&cli_context.tag_prefix).unwrap();
    println!("Found tag '{}', will use that as base", latest.name);

    let relevant_commits = git_context.get_commits_since_tag(latest);

    if relevant_commits.len() != 0 {
        println!("Found the following relevant commits:");
        relevant_commits.iter().for_each(|commit| {
            println!("commit: {:?}", commit);
        });

        let bumplevel = calc_bumplevel(&relevant_commits);

        let mut project = Project::new(&cli_context.path);
        if project.read_project_version() {
            let mut version = SemanticVersion::new(&project.version_string)
                .expect("Failed to parse version string");
            version.bump(bumplevel);

            println!(
                "bump level: {:?} => next version: {}",
                bumplevel,
                version.to_string()
            );
        } else {
            panic!("Failed to find a version string");
        }
    } else {
        println!("Found no commits since the last tag")
    }
}
