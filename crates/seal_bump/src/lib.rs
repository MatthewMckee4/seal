use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use semver::Prerelease;
use thiserror::Error;

mod bump;

pub use bump::calculate_version_file_changes;
pub use semver::Version;

/// Pre-release identifier type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreReleaseType {
    /// Alpha pre-release (e.g., 1.0.0-alpha.1)
    Alpha,
    /// Beta pre-release (e.g., 1.0.0-beta.1)
    Beta,
    /// Release Candidate (e.g., 1.0.0-rc.1)
    Rc,
}

impl fmt::Display for PreReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alpha => write!(f, "alpha"),
            Self::Beta => write!(f, "beta"),
            Self::Rc => write!(f, "rc"),
        }
    }
}

/// Represents a version bump operation, either an explicit version or a bump type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionBump {
    /// Bump the major version (X.0.0)
    Major,
    /// Bump the minor version (x.Y.0)
    Minor,
    /// Bump the patch version (x.y.Z)
    Patch,

    /// Bump major and create pre-release (e.g., 1.2.3 -> 2.0.0-alpha.1)
    MajorPreRelease(PreReleaseType),
    /// Bump minor and create pre-release (e.g., 1.2.3 -> 1.3.0-beta.1)
    MinorPreRelease(PreReleaseType),
    /// Bump patch and create pre-release (e.g., 1.2.3 -> 1.2.4-rc.1)
    PatchPreRelease(PreReleaseType),

    /// Bump pre-release number (e.g., 1.0.0-alpha.1 -> 1.0.0-alpha.2)
    PreRelease(PreReleaseType),

    /// Set an explicit version (e.g., "1.2.3" or "1.2.3-alpha.1")
    Explicit(String),
}

/// Errors that can occur when parsing a version bump argument.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VersionBumpError {
    /// The provided version bump argument is invalid
    #[error(
        "invalid version bump: '{0}'. Expected 'major', 'minor', 'patch', 'alpha', 'beta', 'rc', combinations like 'major-alpha', or a semantic version like '1.2.3'"
    )]
    InvalidBump(String),

    /// The provided explicit version is malformed
    #[error(
        "malformed version: '{0}'. Expected format 'X.Y.Z' where X, Y, and Z are non-negative integers"
    )]
    MalformedVersion(String),

    /// The provided explicit version is a version prior to the current version
    #[error("explicit version '{new}' is prior to the current version '{current}'")]
    ExplicitVersionPrior { current: String, new: String },

    /// The provided explicit version is the same as the current version
    #[error("explicit version '{new}' is the same as the current version '{current}'")]
    ExplicitVersionSame { current: String, new: String },
}

impl FromStr for VersionBump {
    type Err = VersionBumpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.to_lowercase();
        let normalized = input.replace(['_', '.'], "-");

        match normalized.as_str() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),

            "alpha" => Ok(Self::PreRelease(PreReleaseType::Alpha)),
            "beta" => Ok(Self::PreRelease(PreReleaseType::Beta)),
            "rc" => Ok(Self::PreRelease(PreReleaseType::Rc)),

            "major-alpha" => Ok(Self::MajorPreRelease(PreReleaseType::Alpha)),
            "major-beta" => Ok(Self::MajorPreRelease(PreReleaseType::Beta)),
            "major-rc" => Ok(Self::MajorPreRelease(PreReleaseType::Rc)),

            "minor-alpha" => Ok(Self::MinorPreRelease(PreReleaseType::Alpha)),
            "minor-beta" => Ok(Self::MinorPreRelease(PreReleaseType::Beta)),
            "minor-rc" => Ok(Self::MinorPreRelease(PreReleaseType::Rc)),

            "patch-alpha" => Ok(Self::PatchPreRelease(PreReleaseType::Alpha)),
            "patch-beta" => Ok(Self::PatchPreRelease(PreReleaseType::Beta)),
            "patch-rc" => Ok(Self::PatchPreRelease(PreReleaseType::Rc)),

            _ => {
                if Version::parse(s).is_ok() {
                    Ok(Self::Explicit(s.to_string()))
                } else {
                    Err(VersionBumpError::InvalidBump(s.to_string()))
                }
            }
        }
    }
}

impl fmt::Display for VersionBump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::MajorPreRelease(pr_type) => write!(f, "major-{pr_type}"),
            Self::MinorPreRelease(pr_type) => write!(f, "minor-{pr_type}"),
            Self::PatchPreRelease(pr_type) => write!(f, "patch-{pr_type}"),
            Self::PreRelease(pr_type) => write!(f, "{pr_type}"),
            Self::Explicit(version) => write!(f, "{version}"),
        }
    }
}

pub fn calculate_new_version(current: &str, bump: &VersionBump) -> anyhow::Result<Version> {
    let mut current_version = Version::parse(current).context("Invalid current version")?;

    match bump {
        VersionBump::Major => {
            current_version.major += 1;
            current_version.minor = 0;
            current_version.patch = 0;
            current_version.pre = Prerelease::EMPTY;
        }
        VersionBump::Minor => {
            current_version.minor += 1;
            current_version.patch = 0;
            current_version.pre = Prerelease::EMPTY;
        }
        VersionBump::Patch => {
            current_version.patch += 1;
            current_version.pre = Prerelease::EMPTY;
        }
        VersionBump::MajorPreRelease(pr_type) => {
            current_version.major += 1;
            current_version.minor = 0;
            current_version.patch = 0;
            current_version.pre = make_prerelease(*pr_type, 1);
        }
        VersionBump::MinorPreRelease(pr_type) => {
            current_version.minor += 1;
            current_version.patch = 0;
            current_version.pre = make_prerelease(*pr_type, 1);
        }
        VersionBump::PatchPreRelease(pr_type) => {
            current_version.patch += 1;
            current_version.pre = make_prerelease(*pr_type, 1);
        }
        VersionBump::PreRelease(pr_type) => {
            let next_number = extract_prerelease_number(&current_version.pre, *pr_type)?;
            current_version.pre = make_prerelease(*pr_type, next_number);
        }
        VersionBump::Explicit(version) => {
            let new_version = Version::parse(version)
                .map_err(|_| VersionBumpError::MalformedVersion(version.clone()))?;

            if new_version < current_version {
                return Err(VersionBumpError::ExplicitVersionPrior {
                    current: current_version.to_string(),
                    new: new_version.to_string(),
                })
                .context("Invalid version bump");
            }

            if new_version == current_version {
                return Err(VersionBumpError::ExplicitVersionSame {
                    current: current_version.to_string(),
                    new: new_version.to_string(),
                })
                .context("Invalid version bump");
            }

            return Ok(new_version);
        }
    }

    Ok(current_version)
}

fn make_prerelease(pr_type: PreReleaseType, number: u64) -> Prerelease {
    Prerelease::new(&format!("{pr_type}.{number}")).expect("Pre release to be valid")
}

fn extract_prerelease_number(
    pre: &Prerelease,
    expected_type: PreReleaseType,
) -> Result<u64, VersionBumpError> {
    if pre.is_empty() {
        return Ok(0);
    }

    let parts: Vec<&str> = pre.as_str().split('.').collect();

    let current_type = parts[0];

    if current_type != expected_type.to_string() {
        return Err(VersionBumpError::InvalidBump(format!(
            "Cannot bump {expected_type} prerelease on a {current_type} version"
        )));
    }

    let current_number = if parts.len() > 1 {
        parts[1].parse::<u64>().map_err(|_| {
            VersionBumpError::MalformedVersion(format!("Invalid prerelease number in: {pre}"))
        })?
    } else {
        0
    };

    Ok(current_number + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_major() {
        assert_eq!("major".parse::<VersionBump>().unwrap(), VersionBump::Major);
        assert_eq!("MAJOR".parse::<VersionBump>().unwrap(), VersionBump::Major);
        assert_eq!("Major".parse::<VersionBump>().unwrap(), VersionBump::Major);
    }

    #[test]
    fn test_parse_minor() {
        assert_eq!("minor".parse::<VersionBump>().unwrap(), VersionBump::Minor);
        assert_eq!("MINOR".parse::<VersionBump>().unwrap(), VersionBump::Minor);
    }

    #[test]
    fn test_parse_patch() {
        assert_eq!("patch".parse::<VersionBump>().unwrap(), VersionBump::Patch);
        assert_eq!("PATCH".parse::<VersionBump>().unwrap(), VersionBump::Patch);
    }

    #[test]
    fn test_parse_prerelease_only() {
        assert_eq!(
            "alpha".parse::<VersionBump>().unwrap(),
            VersionBump::PreRelease(PreReleaseType::Alpha)
        );
        assert_eq!(
            "beta".parse::<VersionBump>().unwrap(),
            VersionBump::PreRelease(PreReleaseType::Beta)
        );
        assert_eq!(
            "rc".parse::<VersionBump>().unwrap(),
            VersionBump::PreRelease(PreReleaseType::Rc)
        );
        assert_eq!(
            "ALPHA".parse::<VersionBump>().unwrap(),
            VersionBump::PreRelease(PreReleaseType::Alpha)
        );
    }

    #[test]
    fn test_parse_major_prerelease() {
        assert_eq!(
            "major-alpha".parse::<VersionBump>().unwrap(),
            VersionBump::MajorPreRelease(PreReleaseType::Alpha)
        );
        assert_eq!(
            "major-beta".parse::<VersionBump>().unwrap(),
            VersionBump::MajorPreRelease(PreReleaseType::Beta)
        );
        assert_eq!(
            "major-rc".parse::<VersionBump>().unwrap(),
            VersionBump::MajorPreRelease(PreReleaseType::Rc)
        );
        assert_eq!(
            "major_alpha".parse::<VersionBump>().unwrap(),
            VersionBump::MajorPreRelease(PreReleaseType::Alpha)
        );
        assert_eq!(
            "major.beta".parse::<VersionBump>().unwrap(),
            VersionBump::MajorPreRelease(PreReleaseType::Beta)
        );
    }

    #[test]
    fn test_parse_minor_prerelease() {
        assert_eq!(
            "minor-alpha".parse::<VersionBump>().unwrap(),
            VersionBump::MinorPreRelease(PreReleaseType::Alpha)
        );
        assert_eq!(
            "minor-beta".parse::<VersionBump>().unwrap(),
            VersionBump::MinorPreRelease(PreReleaseType::Beta)
        );
        assert_eq!(
            "minor-rc".parse::<VersionBump>().unwrap(),
            VersionBump::MinorPreRelease(PreReleaseType::Rc)
        );
        assert_eq!(
            "patch-rc".parse::<VersionBump>().unwrap(),
            VersionBump::PatchPreRelease(PreReleaseType::Rc)
        );
    }

    #[test]
    fn test_parse_patch_prerelease() {
        assert_eq!(
            "patch-alpha".parse::<VersionBump>().unwrap(),
            VersionBump::PatchPreRelease(PreReleaseType::Alpha)
        );
        assert_eq!(
            "patch-beta".parse::<VersionBump>().unwrap(),
            VersionBump::PatchPreRelease(PreReleaseType::Beta)
        );
        assert_eq!(
            "patch-rc".parse::<VersionBump>().unwrap(),
            VersionBump::PatchPreRelease(PreReleaseType::Rc)
        );
    }

    #[test]
    fn test_parse_explicit_prerelease_version() {
        assert_eq!(
            "1.2.3-alpha.1".parse::<VersionBump>().unwrap(),
            VersionBump::Explicit("1.2.3-alpha.1".to_string())
        );
        assert_eq!(
            "0.1.0-beta.2".parse::<VersionBump>().unwrap(),
            VersionBump::Explicit("0.1.0-beta.2".to_string())
        );
        assert_eq!(
            "2.0.0-rc.1".parse::<VersionBump>().unwrap(),
            VersionBump::Explicit("2.0.0-rc.1".to_string())
        );
    }

    #[test]
    fn test_parse_explicit_version() {
        assert_eq!(
            "1.2.3".parse::<VersionBump>().unwrap(),
            VersionBump::Explicit("1.2.3".to_string())
        );
        assert_eq!(
            "0.0.1".parse::<VersionBump>().unwrap(),
            VersionBump::Explicit("0.0.1".to_string())
        );
        assert_eq!(
            "10.20.30".parse::<VersionBump>().unwrap(),
            VersionBump::Explicit("10.20.30".to_string())
        );
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(
            "invalid".parse::<VersionBump>().unwrap_err(),
            VersionBumpError::InvalidBump("invalid".to_string())
        );
        assert_eq!(
            "1.2".parse::<VersionBump>().unwrap_err(),
            VersionBumpError::InvalidBump("1.2".to_string())
        );
        assert_eq!(
            "1.2.3.4".parse::<VersionBump>().unwrap_err(),
            VersionBumpError::InvalidBump("1.2.3.4".to_string())
        );
        assert_eq!(
            "a.b.c".parse::<VersionBump>().unwrap_err(),
            VersionBumpError::InvalidBump("a.b.c".to_string())
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(VersionBump::Major.to_string(), "major");
        assert_eq!(VersionBump::Minor.to_string(), "minor");
        assert_eq!(VersionBump::Patch.to_string(), "patch");
        assert_eq!(
            VersionBump::PreRelease(PreReleaseType::Alpha).to_string(),
            "alpha"
        );
        assert_eq!(
            VersionBump::MajorPreRelease(PreReleaseType::Beta).to_string(),
            "major-beta"
        );
        assert_eq!(
            VersionBump::MinorPreRelease(PreReleaseType::Rc).to_string(),
            "minor-rc"
        );

        assert_eq!(
            VersionBump::PatchPreRelease(PreReleaseType::Alpha).to_string(),
            "patch-alpha"
        );
        assert_eq!(
            VersionBump::Explicit("1.2.3".to_string()).to_string(),
            "1.2.3"
        );
        assert_eq!(
            VersionBump::Explicit("1.2.3-alpha.1".to_string()).to_string(),
            "1.2.3-alpha.1"
        );
    }

    #[test]
    fn test_prerelease_type_display() {
        assert_eq!(PreReleaseType::Alpha.to_string(), "alpha");
        assert_eq!(PreReleaseType::Beta.to_string(), "beta");
        assert_eq!(PreReleaseType::Rc.to_string(), "rc");
    }

    #[test]
    fn test_calculate_version_major() {
        assert_eq!(
            calculate_new_version("1.2.3", &VersionBump::Major).unwrap(),
            Version::new(2, 0, 0)
        );
        assert_eq!(
            calculate_new_version("0.5.10", &VersionBump::Major).unwrap(),
            Version::new(1, 0, 0)
        );
    }

    #[test]
    fn test_calculate_version_minor() {
        assert_eq!(
            calculate_new_version("1.2.3", &VersionBump::Minor).unwrap(),
            Version::new(1, 3, 0)
        );
        assert_eq!(
            calculate_new_version("2.0.0", &VersionBump::Minor).unwrap(),
            Version::new(2, 1, 0)
        );
    }

    #[test]
    fn test_calculate_version_patch() {
        assert_eq!(
            calculate_new_version("1.2.3", &VersionBump::Patch).unwrap(),
            Version::new(1, 2, 4)
        );
        assert_eq!(
            calculate_new_version("1.0.0", &VersionBump::Patch).unwrap(),
            Version::new(1, 0, 1)
        );
    }

    #[test]
    fn test_calculate_version_major_prerelease() {
        assert_eq!(
            calculate_new_version(
                "1.2.3",
                &VersionBump::MajorPreRelease(PreReleaseType::Alpha)
            )
            .unwrap(),
            Version::parse("2.0.0-alpha.1").unwrap()
        );
        assert_eq!(
            calculate_new_version("0.5.0", &VersionBump::MajorPreRelease(PreReleaseType::Beta))
                .unwrap(),
            Version::parse("1.0.0-beta.1").unwrap()
        );
        assert_eq!(
            calculate_new_version("1.9.9", &VersionBump::MajorPreRelease(PreReleaseType::Rc))
                .unwrap(),
            Version::parse("2.0.0-rc.1").unwrap()
        );
    }

    #[test]
    fn test_calculate_version_minor_prerelease() {
        assert_eq!(
            calculate_new_version(
                "1.2.3",
                &VersionBump::MinorPreRelease(PreReleaseType::Alpha)
            )
            .unwrap(),
            Version::parse("1.3.0-alpha.1").unwrap()
        );
        assert_eq!(
            calculate_new_version("2.0.0", &VersionBump::MinorPreRelease(PreReleaseType::Beta))
                .unwrap(),
            Version::parse("2.1.0-beta.1").unwrap()
        );
    }

    #[test]
    fn test_calculate_version_patch_prerelease() {
        assert_eq!(
            calculate_new_version(
                "1.2.3",
                &VersionBump::PatchPreRelease(PreReleaseType::Alpha)
            )
            .unwrap(),
            Version::parse("1.2.4-alpha.1").unwrap()
        );
        assert_eq!(
            calculate_new_version("1.0.0", &VersionBump::PatchPreRelease(PreReleaseType::Rc))
                .unwrap(),
            Version::parse("1.0.1-rc.1").unwrap()
        );
    }

    #[test]
    fn test_calculate_version_prerelease_bump() {
        assert_eq!(
            calculate_new_version(
                "1.2.3-alpha.1",
                &VersionBump::PreRelease(PreReleaseType::Alpha)
            )
            .unwrap(),
            Version::parse("1.2.3-alpha.2").unwrap()
        );
        assert_eq!(
            calculate_new_version(
                "2.0.0-beta.5",
                &VersionBump::PreRelease(PreReleaseType::Beta)
            )
            .unwrap(),
            Version::parse("2.0.0-beta.6").unwrap()
        );
        assert_eq!(
            calculate_new_version("1.5.0-rc.1", &VersionBump::PreRelease(PreReleaseType::Rc))
                .unwrap(),
            Version::parse("1.5.0-rc.2").unwrap()
        );
    }

    #[test]
    fn test_calculate_version_prerelease_mismatch() {
        let result = calculate_new_version(
            "1.2.3-alpha.1",
            &VersionBump::PreRelease(PreReleaseType::Beta),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot bump beta prerelease on a alpha version")
        );
    }

    #[test]
    fn test_calculate_version_explicit() {
        assert_eq!(
            calculate_new_version("1.2.3", &VersionBump::Explicit("3.0.0".to_string())).unwrap(),
            Version::parse("3.0.0").unwrap()
        );
        assert_eq!(
            calculate_new_version("1.2.3", &VersionBump::Explicit("2.0.0-beta.1".to_string()))
                .unwrap(),
            Version::parse("2.0.0-beta.1").unwrap()
        );
    }

    #[test]
    fn test_calculate_version_invalid() {
        let result =
            calculate_new_version("1.2.3", &VersionBump::Explicit("1.1.1.1.1".to_string()));
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error.downcast_ref::<VersionBumpError>(),
            Some(VersionBumpError::MalformedVersion(_))
        ));
    }
}
