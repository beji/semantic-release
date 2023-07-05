use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use color_eyre::eyre::{self, WrapErr};
use console::style;
use tempfile::NamedTempFile;
use tracing::{debug, info, span, warn, Level};

use crate::git::BumpLevel;
use crate::project::load_versionfile;
use crate::{cli::CliContext, git::calc_bumplevel, semver::SemanticVersion};

mod cli;
mod config;
mod git;
mod init;
mod project;
mod semver;

fn run_command(executable: &str, cwd: &PathBuf, args: Vec<String>) -> eyre::Result<Output> {
    let mut command = Command::new(executable);
    command.args(args).current_dir(cwd);
    let program = command.get_program().to_str().unwrap();
    let args = command.get_args();
    debug!("Executing {:?} {:?}", program, args);
    let output = command
        .output()
        .context("Failed to get output from command")?;
    Ok(output)
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cli_context = CliContext::new().expect("Failed to build CLI Context");

    tracing_subscriber::fmt::fmt()
        .without_time()
        .with_max_level(cli_context.log_level)
        .with_target(false)
        .init();

    if cli_context.init {
        init::init_project(&cli_context).wrap_err("Failed to initialize a new config file")?;
    } else {
        let config = config::Config::from_path(&cli_context.path)
            .context("Failed to build configuration")?;

        let path = Path::new(&cli_context.path);
        let path = path.parent().unwrap();
        let path = fs::canonicalize(path)?;
        let subpath = path.join(&config.subpath);

        let mut semver = SemanticVersion::new();
        let mut commit_changes = false;

        config.files.iter().try_for_each(|file| -> eyre::Result<()> {
        let span = span!(Level::TRACE, "file", file = &file.path);
        let _guard = span.enter();
        let filename = &file.path;
        info!("Handling file {}", style(&filename).bold());
        let filepath = subpath.join(filename);
        let project_type = &file.project_type;

        debug!("Path: {}", &filepath.display());
        let mut version_file = load_versionfile(&filepath, file).context("Failed to build internal representation of project file")?;

        let version = version_file.read_version()
            .context("Failed to read version from project file")?;

        debug!("Version: {}", version);

        info!("Fetching tags");
        let args = vec![
            "tag".to_owned(),
            "--list".to_owned(),
            format!("{}{}", &config.tagprefix, &version),
            "--sort=-creatordate".to_owned(),
        ];
        let tags = run_command("git", &subpath, args).context("Failed to get git tags")?;

        let tags: Vec<&str> = std::str::from_utf8(&tags.stdout)?
            .split('\n')
            .filter(|line| !line.is_empty())
            .collect();
        if tags.is_empty() {
            warn!("Could not find a tag matching {}", &config.tagprefix);
            warn!("Stopping execution");
        } else {
            debug!("matching tags: {:?}", tags);
            let last_tag = *tags.first().unwrap();
            info!(
                "Found {} as the latest relevant tag",
                style(last_tag).bold()
            );

            info!("Fetching relevant commits");
            let args = vec![
                "rev-list".to_owned(),
                format!("{}..HEAD", last_tag),
                "--format=%B".to_owned(),
                ".".to_owned(),
            ];
            let commits = run_command("git", &subpath, args).context("Failed to get git commits")?;
            let commits: Vec<&str> = std::str::from_utf8(&commits.stdout)?
                .split('\n')
                .filter(|line| !line.is_empty())
                .collect();
            debug!("Found {:?} as relevant commits", commits);

            if commits.is_empty() {
                info!("No relevant commits found. Not doing anything");
                return Ok(());
            }

            info!("Calculating Bumplevel");
            let bumplevel = calc_bumplevel(&commits);
            info!("Bumplevel: {:?}", style(&bumplevel).bold());

            if bumplevel == BumpLevel::None {
                commit_changes = false;
            }

            semver.set_version(&version)
                .context("Failed to parse version into a semantic version")?;
            semver.bump(bumplevel);

            info!("Parsing {:?} with type {:?}", filepath, project_type);
            let json =
                version_file.update_project(&semver).context("Failed to update file JSON")?;
            if cli_context.dryrun {
                info!("Dry run is active, not writing the file");
                debug!("Would write {}", json);
            } else {
                let mut file_handle = NamedTempFile::new().context("Failed to create temporary file")?;
                file_handle.write_all(json.as_bytes()).context("Failed to write to temporary file, maybe the user is lacking the necessary permission")?;
                let path = &file_handle.path().clone();
                info!("Successfully updated the project file");
                debug!("Moving temporay file {:?} to {:?}", &path, &filepath);
                fs::copy(path, &filepath).context("Failed to copy from temporary file to target")?;
            }
            info!(
                "Adding {} to the git commit",
                style(&filepath.display()).bold()
            );
            // TODO: There must be a better way than format!
            let args = vec!["add".to_owned(), format!("{}", filename)];
            if cli_context.dryrun {
                info!("Dry run is active, not adding the file");
                debug!(
                    "Would run git with the arguments {:?} from the directory {:?}",
                    args, &subpath
                );
            } else {
                run_command("git", &subpath, args).context("Failed to execute git add")?;
            }
            commit_changes = true;
        }

        eyre::Result::Ok(())
    })?;

        if commit_changes {
            info!("Doing the git commit");
            let args = vec![
                "commit".to_string(),
                "-m".to_string(),
                format!("[Semantic release]: Release {}", &semver.to_string()),
            ];
            if cli_context.dryrun {
                info!("Dry run is active, not commiting anything");
                debug!(
                    "Would run git with the arguments {:?} from the directory {:?}",
                    args, &subpath
                );
            } else {
                run_command("git", &subpath, args).context("Failed to execute git commit")?;
            }

            // TODO: Maybe make tagging optional?
            let tag = format!("{}{}", &config.tagprefix, &semver.to_string());
            info!("Tagging the release with tag {}", style(&tag).bold());
            let args = vec!["tag".to_string(), tag];
            if cli_context.dryrun {
                info!("Dry run is active, not tagging anything");
                debug!(
                    "Would run git with the arguments {:?} from the directory {:?}",
                    args, &subpath
                );
            } else {
                run_command("git", &subpath, args).context("Failed to execute git tag")?;
            }

            info!(
                "All done! keep in mind that this doesn't do a {}",
                style("git push").bold()
            );
        } else {
            info!("Nothing to change");
        }
    }
    Ok(())
}
