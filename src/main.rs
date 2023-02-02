#[macro_use]
extern crate lazy_static;

use anyhow::{bail, Context};
use console::style;
use git::BumpLevel;
use tracing::{debug, info};

use crate::cli::CliContext;
use crate::git::{calc_bumplevel, GitContext};
use crate::semver::SemanticVersion;

mod cli;
mod git;
mod semver;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .init();

    let cli_context = CliContext::new().expect("Failed to build CLI Context");

    info!(
        "Looking for a git repo at (or above) {}",
        style(&cli_context.path).bold()
    );

    let git_context = GitContext::new(cli_context.path.as_str());

    let latest = git_context
        .get_latest_tag(cli_context.tag_prefix.as_str())
        .unwrap();
    info!(
        "Found tag {}, will use that as base",
        style(&latest.name).bold()
    );

    let relevant_commits = git_context.get_commits_since_tag(latest);

    if !relevant_commits.is_empty() {
        debug!("Found the following relevant commits:");
        relevant_commits
            .iter()
            .for_each(|commit| debug!("commit: {:?}", commit));

        let bumplevel = calc_bumplevel(
            &relevant_commits,
            &cli_context.patchtokens,
            &cli_context.minortokens,
        );

        // let mut project = new_project(&cli_context.path);

        // if project.get_project_type() == ProjectType::Unknown {
        //     bail!("The project type isn't currently implemented");
        // }
        // if project.read_project_version() {
        //     let mut version = SemanticVersion::new(project.get_version_string())
        //         .context("Failed to parse version string")?;
        //     version.bump(bumplevel);

        //     info!(
        //         "bump level: {:?} => next version: {}",
        //         style(&bumplevel).bold(),
        //         style(&version.to_string()).bold()
        //     );

        //     if cli_context.dryrun {
        //         info!("Dry run activated, not proceeding any further");
        //     } else if bumplevel == BumpLevel::None {
        //         info!("No relevant tags found that have a matching format; nothing to do here");
        //     } else {
        //         project.update_project_version_file(&version);

        //         let commit_id = git_context.commit_release(&version, project.get_project_file());
        //         git_context.tag_release(cli_context.tag_prefix.as_str(), &version, &commit_id);
        //     }
        // } else {
        //     bail!("Failed to find a version string");
        // }
    } else {
        info!("Found no commits since the last tag");
    }
    Ok(())
}
