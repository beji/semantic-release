use std::fmt;

use color_eyre::eyre;
use console::style;
use tracing::{debug, info, instrument, trace};

use crate::git::BumpLevel;

#[derive(Debug)]
pub struct SemanticVersion {
    major: usize,
    minor: usize,
    patch: usize,
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl SemanticVersion {
    pub fn new() -> Self {
        SemanticVersion {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
    #[instrument(level = "trace", name = "SemanticVersion::set_version")]
    pub fn set_version(&mut self, version_str: &str) -> eyre::Result<()> {
        debug!("Trying to parse: {}", version_str);
        let split = version_str.split('.');
        if split.clone().count() < 3 {
            eyre::bail!(
                "Failed to split the given string into exactly three parts. Found too few parts",
            );
        }
        let mut too_many_parts = false;
        for (i, el) in split.enumerate() {
            match i {
                0 => self.major = el.parse::<usize>().unwrap(),
                1 => self.minor = el.parse::<usize>().unwrap(),
                2 => self.patch = el.parse::<usize>().unwrap(),
                _ => too_many_parts = true,
            }
        }
        if too_many_parts {
            eyre::bail!(
                "Failed to split the given string into exactly three parts. Found too many parts",
            );
        }
        Ok(())
    }

    #[instrument(level = "trace", name = "SemanticVersion::bump")]
    pub fn bump(&mut self, bumplevel: BumpLevel) {
        debug!("bumping version: {}", self);
        match bumplevel {
            BumpLevel::Patch => {
                trace!("Patch level bump");
                self.patch += 1;
            }
            BumpLevel::Minor => {
                trace!("Minor level bump");
                self.minor += 1;
                self.patch = 0;
            }
            BumpLevel::Major => {
                trace!("Major level bump");
                self.major += 1;
                self.minor = 0;
                self.patch = 0;
            }
            BumpLevel::None => {
                trace!("No bump happening");
            }
        };
        info!("Next version: {}", style(self).bold());
    }
}

impl Default for SemanticVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::semver::*;

    #[test]
    fn to_string() {
        let n = SemanticVersion {
            major: 1,
            minor: 2,
            patch: 3,
        };
        assert_eq!(n.to_string(), "1.2.3");
    }
}
