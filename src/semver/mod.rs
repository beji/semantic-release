use std::fmt;

use crate::git::BumpLevel;

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
    pub fn new(version_str: &str) -> Result<SemanticVersion, &'static str> {
        println!("Trying to parse: {}", version_str);
        let split = version_str.split(".");
        if split.clone().count() < 3 {
            return Err(
                "Failed to split the given string into exactly three parts. Found too few parts",
            );
        }
        let mut version = SemanticVersion {
            major: 0,
            minor: 0,
            patch: 0,
        };
        let mut too_many_parts = false;
        for (i, el) in split.enumerate() {
            match i {
                0 => version.major = el.parse::<usize>().unwrap(),
                1 => version.minor = el.parse::<usize>().unwrap(),
                2 => version.patch = el.parse::<usize>().unwrap(),
                _ => too_many_parts = true,
            }
        }
        if too_many_parts {
            return Err(
                "Failed to split the given string into exactly three parts. Found too many parts",
            );
        }
        Ok(version)
    }

    pub fn bump(&mut self, bumplevel: BumpLevel) {
        match bumplevel {
            BumpLevel::Patch => {
                self.patch += 1;
            }
            BumpLevel::Minor => {
                self.minor += 1;
            }
            BumpLevel::Major => {
                self.major += 1;
            }
            BumpLevel::None => (),
        };
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

    #[test]
    fn new() {
        let a = SemanticVersion::new("1.2.3");
        let a_sv = a.as_ref().unwrap();
        assert!(a.is_ok());
        assert_eq!(a_sv.to_string(), "1.2.3");
        let b = SemanticVersion::new("0.0.0").unwrap();
        assert_eq!(b.to_string(), "0.0.0");
    }

    #[test]
    fn bump_major() {
        let mut a = SemanticVersion::new("1.2.3").unwrap();
        a.bump(BumpLevel::Major);
        assert_eq!(a.to_string(), "2.2.3");
    }

    #[test]
    fn bump_minor() {
        let mut a = SemanticVersion::new("1.2.3").unwrap();
        a.bump(BumpLevel::Minor);
        assert_eq!(a.to_string(), "1.3.3");
    }

    #[test]
    fn bump_patch() {
        let mut a = SemanticVersion::new("1.2.3").unwrap();
        a.bump(BumpLevel::Patch);
        assert_eq!(a.to_string(), "1.2.4");
    }

    #[test]
    fn bump_none() {
        let mut a = SemanticVersion::new("1.2.3").unwrap();
        a.bump(BumpLevel::None);
        assert_eq!(a.to_string(), "1.2.3");
    }
}
