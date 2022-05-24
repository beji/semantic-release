use std::env;
use std::path::Path;

use crate::git::GitContext;
use crate::semver::SemanticVersion;

mod git;
mod semver;

fn main() {
    let v = SemanticVersion::new("1.2.3").unwrap();
    println!("Hello, world! {}", v.to_string());

    let mut args = env::args();
    //move past the command
    args.next();
    let path_str = args.next().unwrap();

    let path = Path::new(&path_str);

    println!("Path {}", path.display());

    let git_context = GitContext::new(path);

    let latest = git_context.get_latest_tag("package-a").unwrap();

    println!("Latest Tag: {:?}", latest);

    git_context.get_commits_since_tag(latest);
}
