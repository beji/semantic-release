#[macro_use]
extern crate lazy_static;

use cli::logger::Logger;
use console::style;
use git::BumpLevel;
use projectparse::new_project;

use crate::cli::CliContext;
use crate::git::{calc_bumplevel, GitContext};
use crate::projectparse::Project;
use crate::projectparse::ProjectType;
use crate::semver::SemanticVersion;

mod cli;
mod git;
mod projectparse;
mod semver;

fn main() {
    let cli_context = CliContext::new().expect("Failed to build CLI Context");
    let logger = Logger::new(cli_context.log_level);

    logger.log_info(format!(
        "Looking for a git repo at (or above) {}",
        style(&cli_context.path).bold()
    ));
    let git_context = GitContext::new(cli_context.path.as_str(), &logger);

    let latest = git_context
        .get_latest_tag(cli_context.tag_prefix.as_str())
        .unwrap();
    logger.log_info(format!(
        "Found tag {}, will use that as base",
        style(&latest.name).bold()
    ));

    let relevant_commits = git_context.get_commits_since_tag(latest);

    if relevant_commits.len() != 0 {
        logger.log_debug("Found the following relevant commits:".to_string());
        relevant_commits
            .iter()
            .for_each(|commit| logger.log_debug(format!("commit: {:?}", commit)));

        let bumplevel = calc_bumplevel(&relevant_commits);

        let mut project = new_project(&cli_context.path, &logger);

        if project.get_project_type() == ProjectType::Unknown {
            panic!("The project type isn't currently implemented");
        }
        if project.read_project_version() {
            let mut version = SemanticVersion::new(project.get_version_string(), &logger)
                .expect("Failed to parse version string");
            version.bump(bumplevel);

            logger.log_info(format!(
                "bump level: {:?} => next version: {}",
                style(&bumplevel).bold(),
                style(&version.to_string()).bold()
            ));

            if cli_context.dryrun {
                logger.log_info("Dry run activated, not proceeding any further".to_string());
            } else {
                if bumplevel == BumpLevel::None {
                    logger.log_info(
                        "No relevant tags found that have a matching format; nothing to do here"
                            .to_string(),
                    );
                } else {
                    project.update_project_version_file(&version);

                    let commit_id =
                        git_context.commit_release(&version, project.get_project_file());
                    git_context.tag_release(cli_context.tag_prefix.as_str(), &version, &commit_id);
                }
            }
        } else {
            panic!("Failed to find a version string");
        }
    } else {
        logger.log_info("Found no commits since the last tag".to_string())
    }
}
