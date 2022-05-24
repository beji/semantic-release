use std::path::Path;

use git2::{Oid, Repository};

pub struct GitContext {
    repo: Repository,
}

#[derive(Clone, Debug)]
pub struct GitTag {
    pub id: Oid,
    pub name: String,
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
        self.repo.tag_foreach(|id, name| {
            let name_string = String::from_utf8_lossy(name);
            if name_string.starts_with(&search_prefix) {
                tags.push(GitTag {
                    id,
                    name: name_string.into_owned(),
                })
            }
            true
        });

        tags.last().cloned()
    }

    pub fn get_commits_since_tag(&self, tag: GitTag) {
        let head = self
            .repo
            .head()
            .expect("Failed to get HEAD")
            .target()
            .expect("Failed to get an id from HEAD");
        let mut revwalk = self.repo.revwalk().expect("Failed to get revwalk thing");
        revwalk
            .push_range(format!("{}..{}", tag.id, head).as_str())
            .expect("Failed to push the start tag id");

        for id in revwalk {
            println!("id: {}", id.unwrap());
        }
    }
}
