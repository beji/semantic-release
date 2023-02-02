use anyhow::{bail, Context};
use console::style;
use git::BumpLevel;
use tracing::{debug, info, trace, warn};

use crate::cli::CliContext;
use crate::git::{calc_bumplevel, GitContext};
use crate::project::{
    find_project_files, read_project_version, update_project_version_file, ProjectFile,
};
use crate::semver::SemanticVersion;

mod cli;
mod git;
mod project;
mod semver;

fn main() -> anyhow::Result<()> {
    debug!("No idea bro");
    let cli_context = CliContext::new().expect("Failed to build CLI Context");

    tracing_subscriber::fmt::fmt()
        .without_time()
        .with_max_level(cli_context.log_level)
        .with_target(false)
        .init();

    info!(
        "Looking for a git repo at (or above) {}",
        style(&cli_context.path).bold()
    );

    let git_context =
        GitContext::new(cli_context.path.as_str()).context("Failed to build a git context")?;

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
            .for_each(|commit| trace!("commit: {:?}", commit));

        let bumplevel = calc_bumplevel(
            &relevant_commits,
            &cli_context.patchtokens,
            &cli_context.minortokens,
        );

        if bumplevel == BumpLevel::None {
            info!("No relevant tags found that have a matching format; nothing to do here");
        } else {
            debug!("Scanning for relevant files");
            let relevant_files = find_project_files(&cli_context.path)
                .context("Failed to get any relevant files from filesystem")?;

            if relevant_files.is_empty() {
                warn!("Could not find any supported project file types");
                bail!("Could not find any supported project file types");
            } else {
                trace!("Found project files");
                if relevant_files.len() > 1 {
                    info!("Found multiple project files");
                    info!("Will try to update them all to the same version");
                }

                let files_with_versions: Vec<(SemanticVersion, usize, ProjectFile)> =
                    relevant_files
                        .iter()
                        .filter_map(|file| {
                            let filename = &file.filename;
                            info!("Handling file {}", filename.display());
                            let version = read_project_version(file.to_owned())
                                .with_context(|| {
                                    format!(
                                        "Failed to extract a version from the project file {}",
                                        filename.display()
                                    )
                                })
                                .ok()?;
                            match version {
                                Some(version) => {
                                    let mut next_version = SemanticVersion::new(&version.version)
                                        .unwrap_or_else(|_| {
                                            panic!(
                                                "Failed to parse a valid semantic version from {}",
                                                version.version
                                            )
                                        });
                                    next_version.bump(bumplevel);
                                    Some((next_version, version.line, file.to_owned()))
                                }
                                None => {
                                    debug!(
                                        "Could not find a version in {}, dropping it",
                                        filename.display()
                                    );
                                    None
                                }
                            }
                        })
                        .collect();

                if cli_context.dryrun {
                    info!("Dry run activated, not proceeding any further");
                } else {
                    info!("Starting file updates now");
                    // Just use the first one we find...
                    let (version_for_release, _, _) = files_with_versions
                        .get(0)
                        .context("Failed to get the first version")?;
                    let version_for_release = version_for_release.clone().to_string();
                    for (version, line, file) in files_with_versions {
                        let filename = &file.filename;
                        let _ = update_project_version_file(&file, version, line)
                            .context("Failed to update a project version file");

                        git_context.add_to_release(filename).with_context(|| {
                            format!(
                                "Failed to add {} to the commit",
                                style(filename.display()).bold()
                            )
                        })?;
                    }
                    let oid = git_context
                        .finish_release(&version_for_release)
                        .context("Failed to create a git commit")?;
                    let _ = git_context
                        .tag_release(&cli_context.tag_prefix, &version_for_release, oid)
                        .context("Failed to tag the commit")?;
                }
            }
        }
    } else {
        info!("Found no commits since the last tag");
    }
    Ok(())
}
