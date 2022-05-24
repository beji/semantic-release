use std::path::Path;

use git2::{DiffFormat, DiffOptions, Oid, Repository};

pub struct GitContext {
    repo: Repository,
}

#[derive(Clone, Debug)]
pub struct GitTag {
    id: Oid,
    pub name: String,
}

#[derive(Debug)]
pub struct GitCommit {
    id: Oid,
    summary: String,
    body: Option<String>,
}

impl GitContext {
    pub fn new(path: &Path) -> GitContext {
        let repo = Repository::discover(path).expect(&format!(
            "Failed to open a git repo at {}, is this a git repo?",
            path.display()
        ));
        GitContext { repo }
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

    pub fn get_commits_since_tag(&self, tag: GitTag, path: &str) -> Vec<GitCommit> {
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
                    if changed_file.starts_with(path) {
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
