//! this is the single integration test, as documented by matklad
//! in <https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html>

pub(crate) mod common;

#[cfg(feature = "integration-test")]
mod bump;

#[cfg(feature = "integration-test")]
mod generate;

mod help;
mod migrate;
mod self_version;
mod validate;
