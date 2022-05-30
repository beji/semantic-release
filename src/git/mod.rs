use std::{fs::canonicalize, path::Path};

use console::style;
use git2::{DiffFormat, DiffOptions, Oid, Repository};

use crate::cli::CliContext;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum BumpLevel {
    None,
    Patch,
    Minor,
    Major,
}

pub struct GitContext<'a> {
    repo: Repository,
    sub_path: String,
    cli_context: &'a CliContext,
}

impl GitContext<'_> {
    pub fn new<'a>(cli_context: &'a CliContext) -> GitContext<'a> {
        let path = Path::new(&cli_context.path);
        let path = canonicalize(&path).expect("Failed to canonicalize input path");

        let repo = Repository::discover(&path).expect(&format!(
            "Failed to open a git repo at {}, is this a git repo?",
            path.as_path().display()
        ));

        let repo_path = repo
            .path()
            .parent()
            .expect("Failed to move up from the discovered repository")
            .to_str()
            .expect("Failed to build string from repo path");

        cli_context.log_info(format!(
            "Found a git repository at {}",
            style(&repo_path).bold()
        ));

        let sub_path = path
            .display()
            .to_string()
            .replace(&format!("{}/", repo_path), "");

        cli_context.log_info(format!(
            "Working with the relative repository path {}",
            style(&sub_path).bold()
        ));

        GitContext {
            repo,
            sub_path,
            cli_context,
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

                diff.print(DiffFormat::NameOnly, |_delta, _hunk, line| {
                    let changed_file = String::from_utf8_lossy(line.content());
                    if changed_file.starts_with(&self.sub_path) {
                        is_relevant = true;
                    }
                    true
                })
                .expect("Failed to print the delta");

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
