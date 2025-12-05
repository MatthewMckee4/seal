use std::fmt;

use serde::Serialize;

/// seal's version.
#[derive(Serialize)]
pub struct VersionInfo {
    /// Name of the package (or "seal" if printing seal's own version)
    pub package_name: Option<String>,
    /// version, such as "0.5.1"
    pub version: String,
}

impl fmt::Display for VersionInfo {
    /// Formatted version information: "<version>[+<commits>] (<commit> <date>)"
    ///
    /// This is intended for consumption by `clap` to provide `uv --version`,
    /// and intentionally omits the name of the package
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;
        Ok(())
    }
}

impl From<VersionInfo> for clap::builder::Str {
    fn from(val: VersionInfo) -> Self {
        val.to_string().into()
    }
}

/// Returns information about seal's version.
pub fn seal_self_version() -> VersionInfo {
    // This version is pulled from Cargo.toml and set by Cargo
    let version = seal_version::version().to_string();

    VersionInfo {
        package_name: Some("seal".to_owned()),
        version,
    }
}

/// Returns just the version string for seal.
pub fn seal_version_string() -> &'static str {
    seal_version::version()
}
