use console::style;
use tracing::{debug, instrument};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum BumpLevel {
    None,
    Patch,
    Minor,
    Major,
}

#[instrument(level = "trace")]
fn summary_to_bumplevel(summary: &str) -> BumpLevel {
    let mut bump_level = BumpLevel::None;
    let boldsummary = style(&summary).bold();
    debug!("checking summary {}", boldsummary);
    if summary.starts_with("fix") {
        debug!(
            "Found a {} indicator in message {}",
            style("patch level").bold(),
            boldsummary
        );
        bump_level = BumpLevel::Patch;
    } else if summary.starts_with("feat") {
        debug!(
            "Found a {} indicator in message {}",
            style("minor level").bold(),
            boldsummary
        );
        bump_level = BumpLevel::Minor;
    } else if summary.contains("BREAKING") {
        debug!(
            "Found a {} indicator in message {}",
            style("breaking change").bold(),
            boldsummary
        );
        bump_level = BumpLevel::Major;
    }
    bump_level
}

pub fn calc_bumplevel(commits: &[&str]) -> BumpLevel {
    let mut bumplevels: Vec<BumpLevel> = commits
        .iter()
        .map(|commit| summary_to_bumplevel(commit))
        .collect();
    bumplevels.sort();
    *bumplevels.last().expect("Failed to get last element from bumplevels list; Most likely calc_bumplevel was called on an empty list")
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
}
