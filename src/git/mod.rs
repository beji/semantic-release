use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};

use console::style;
use git2::{DiffFormat, DiffOptions, ObjectType, Oid, Repository, Signature};

use crate::{cli::logger::Logger, semver::SemanticVersion};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum BumpLevel {
    None,
    Patch,
    Minor,
    Major,
}

pub struct GitContext<'a> {
    repo: Repository,
    sub_path: Option<String>,
    logger: &'a Logger,
}

impl GitContext<'_> {
    pub fn new<'a>(path: &str, logger: &'a Logger) -> GitContext<'a> {
        let path = Path::new(path);
        // let path = canonicalize(&path).expect("Failed to canonicalize input path");

        let repo = Repository::discover(&path).expect(&format!(
            "Failed to open a git repo at {}, is this a git repo?",
            path.display()
        ));

        logger.log_info(format!(
            "Found a git repository at {}",
            style(repo.path().display()).bold()
        ));

        let sub_path = path_relative_to_repo(&repo, path);

        match &sub_path {
            Some(sub_path) => logger.log_info(format!(
                "Working with the relative repository path {}",
                style(&sub_path).bold()
            )),
            None => logger.log_info("Working from the repository root".to_string()),
        }

        GitContext {
            repo,
            sub_path,
            logger,
        }
    }

    pub fn get_latest_tag(&self, prefix: &str) -> Option<GitTag> {
        let search_prefix = "refs/tags/".to_owned() + prefix;
        let mut tags: Vec<GitTag> = Vec::new();
        self.repo
            .tag_foreach(|id, name| {
                let name_string = String::from_utf8_lossy(name);
                if name_string.starts_with(&search_prefix) {
                    tags.push(GitTag {
                        id,
                        name: name_string.into_owned(),
                    })
                }
                true
            })
            .expect("Failed to loop over tags");

        tags.last().cloned()
    }

    pub fn get_commits_since_tag(&self, tag: GitTag) -> Vec<GitCommit> {
        let head = self
            .repo
            .head()
            .expect("Failed to get HEAD")
            .target()
            .expect("Failed to get an id from HEAD");

        // Revwalk is used to iter over commits, in this case from tag.id to head
        let mut revwalk = self.repo.revwalk().expect("Failed to get revwalk thing");
        revwalk
            .push_range(format!("{}..{}", tag.id, head).as_str())
            .expect("Failed to push the start tag id");

        revwalk
            .into_iter()
            .filter_map(|id| {
                let id = id.expect("Failed to get an id from revwalk entry");
                let commit = self
                    .repo
                    .find_commit(id)
                    .expect(format!("Failed to find commit from id {}", id).as_str());
                let tree = commit
                    .tree()
                    .expect(format!("Failed to get tree from id {}", id).as_str());
                let parent = commit
                    .parent(0)
                    .expect(format!("Failed to find parent to commit {}", id).as_str())
                    .tree()
                    .expect("Failed to get tree from parent");
                // TODO: Maybe diffing with index would be better, no idea
                let diff = self
                    .repo
                    .diff_tree_to_tree(Some(&parent), Some(&tree), Some(&mut DiffOptions::new()))
                    .expect(format!("Failed to build diff for commit {}", id).as_str());

                let summary = commit
                    .summary()
                    .expect(format!("Failed to extract summary for commit {}", id).as_str());

                let mut is_relevant = false;

                match &self.sub_path {
                    // Working with a subdir, need to check every commit
                    Some(sub_path) => diff
                        .print(DiffFormat::NameOnly, |_delta, _hunk, line| {
                            let changed_file = String::from_utf8_lossy(line.content());
                            if changed_file.starts_with(sub_path.as_str()) {
                                is_relevant = true;
                            }
                            true
                        })
                        .expect("Failed to print the delta"),
                    // Working from the root dir, every commit is relevant by definition
                    None => is_relevant = true,
                }

                let body = match commit.body() {
                    Some(body) => Some(body.to_owned()),
                    None => None,
                };

                if is_relevant {
                    Some(GitCommit {
                        id,
                        summary: summary.to_owned(),
                        body,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn commit_release(&self, version: &SemanticVersion, project_file: &PathBuf) -> Oid {
        let version = version.to_string();
        let sig = self
            .repo
            .signature()
            .expect("Failed to get signature from repo");
        let mut index = self.repo.index().expect("Failed to get repo index");
        let project_file = Path::new(project_file);
        let project_file = path_relative_to_repo(&self.repo, &project_file)
            .expect("Failed to build a relative path for the project file");

        index
            .add_path(Path::new(&project_file))
            .expect("Failed to add path to index");
        index.write().expect("Failed to write index");
        let tree_oid = index.write_tree().expect("Failed to write git tree");
        let tree = self
            .repo
            .find_tree(tree_oid)
            .expect("Failed to get tree by oid");

        let parent_commit = self.repo.head().unwrap().peel_to_commit().unwrap();

        let result_id = self
            .repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                format!("[Semantic release]: Release {}", version.as_str()).as_str(),
                &tree,
                &[&parent_commit],
            )
            .expect("Failed to commit changes");

        self.logger
            .log_debug(format!("Created git commit {}", result_id));

        result_id
    }

    pub fn tag_release(&self, tag_prefix: &str, version: &SemanticVersion, oid: &Oid) {
        let version = version.to_string();
        let object = self
            .repo
            .find_object(*oid, Some(ObjectType::Commit))
            .expect("Failed to find git object by oid");
        let tag_name = format!("{}{}", tag_prefix, version.as_str());
        self.repo
            .tag_lightweight(tag_name.as_str(), &object, false)
            .expect("Failed to tag release");

        self.logger.log_info(format!("Created tag {}", tag_name));
    }
}

#[derive(Clone, Debug)]
pub struct GitTag {
    id: Oid,
    pub name: String,
}

#[derive(Debug)]
pub struct GitCommit {
    id: Oid,
    pub summary: String,
    body: Option<String>,
}

impl GitCommit {
    pub fn for_test(summary: String, body: Option<String>) -> GitCommit {
        let id = Oid::from_str("1").unwrap();
        GitCommit { id, summary, body }
    }

    pub fn to_bumplevel(&self) -> BumpLevel {
        let summary = &self.summary;
        let body = &self.body;

        match body {
            Some(body) => {
                if body.contains("BREAKING CHANGE:") {
                    BumpLevel::Major
                } else {
                    summary_to_bumplevel(&summary)
                }
            }
            None => summary_to_bumplevel(&summary),
        }
    }
}

fn summary_to_bumplevel(summary: &str) -> BumpLevel {
    if summary.starts_with("fix") {
        BumpLevel::Patch
    } else if summary.starts_with("feat") {
        BumpLevel::Minor
    } else {
        BumpLevel::None
    }
}

pub fn calc_bumplevel(commits: &Vec<GitCommit>) -> BumpLevel {
    let mut bumplevels: Vec<BumpLevel> =
        commits.iter().map(|commit| commit.to_bumplevel()).collect();
    bumplevels.sort();
    bumplevels.last().expect("Failed to get last element from bumplevels list; Most likely calc_bumplevel was called on an empty list").clone()
}

pub fn path_relative_to_repo(repo: &Repository, path: &Path) -> Option<String> {
    let path = canonicalize(&path).expect("Failed to canonicalize input path");

    let repo_path = repo
        .path()
        .parent()
        .expect("Failed to move up from the discovered repository")
        .to_str()
        .expect("Failed to build string from repo path");

    println!("repo_path {}", repo_path);

    println!("path.display {}", path.display());

    // Equal paths means that the path given here is equal to the git project root
    if repo_path == path.display().to_string() {
        None
    } else {
        let sub_path = path
            .display()
            .to_string()
            .replace(&format!("{}/", repo_path), "");
        Some(sub_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::git::*;

    #[test]
    fn bumplevel_comparison() {
        assert!(BumpLevel::None < BumpLevel::Patch);
        assert!(BumpLevel::Patch < BumpLevel::Minor);
        assert!(BumpLevel::Minor < BumpLevel::Major);
    }

    #[test]
    fn calc_bumplevel_minor() {
        let input: Vec<GitCommit> = vec![
            GitCommit::for_test("fix".to_string(), None),
            GitCommit::for_test("feat".to_string(), None),
            GitCommit::for_test("fix".to_string(), None),
        ];

        let result = calc_bumplevel(&input);
        assert_eq!(result, BumpLevel::Minor);
    }

    #[test]
    fn calc_bumplevel_major() {
        let input: Vec<GitCommit> = vec![
            GitCommit::for_test("fix".to_string(), Some("BREAKING CHANGE:".to_string())),
            GitCommit::for_test("feat".to_string(), None),
            GitCommit::for_test("fix".to_string(), None),
        ];

        let result = calc_bumplevel(&input);
        assert_eq!(result, BumpLevel::Major);
    }
}
