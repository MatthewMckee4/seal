use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn bump_major() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["Cargo.toml", "package.json", "VERSION"],
            "Release v{version}",
            "release/v{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            r#"[package]
name = "test"
version = "1.2.3"
"#,
        )
        .unwrap();

    context
        .root
        .child("package.json")
        .write_str(r#"{"name": "test", "version": "1.2.3"}"#)
        .unwrap();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0
    Creating branch: release/v2.0.0
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "release/v2.0.0");
    assert_eq!(context.git_last_commit_message(), "Release v2.0.0");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "test"
    version = "2.0.0"
    "###);

    insta::assert_snapshot!(context.read_file("package.json"), @r###"{"name": "test", "version": "2.0.0"}"###);

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "2.0.0""###);
}

#[test]
fn bump_minor() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["Cargo.toml"],
            "chore: bump to {version}",
            "releases/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            r#"[package]
name = "test"
version = "1.2.3"
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.3.0
    Creating branch: releases/1.3.0
    Updating version files...
    Committing changes...
    Successfully bumped to 1.3.0

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "releases/1.3.0");
    assert_eq!(context.git_last_commit_message(), "chore: bump to 1.3.0");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "test"
    version = "1.3.0"
    "###);
}

#[test]
fn bump_patch() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "2.1.5",
            &["VERSION.txt"],
            "Version {version}",
            "bump/{version}",
            "{version}",
        )
        .init_git();

    context
        .root
        .child("VERSION.txt")
        .write_str(r#"version = "2.1.5""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.1.5 to 2.1.6
    Creating branch: bump/2.1.6
    Updating version files...
    Committing changes...
    Successfully bumped to 2.1.6

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "bump/2.1.6");
    assert_eq!(context.git_last_commit_message(), "Version 2.1.6");

    insta::assert_snapshot!(context.read_file("VERSION.txt"), @r###"version = "2.1.6""###);
}

#[test]
fn bump_major_alpha() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["version.txt"],
            "Bump {version}",
            "rel/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("version.txt")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major-alpha").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0-alpha.1
    Creating branch: rel/2.0.0-alpha.1
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-alpha.1

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "rel/2.0.0-alpha.1");
    assert_eq!(context.git_last_commit_message(), "Bump 2.0.0-alpha.1");

    insta::assert_snapshot!(context.read_file("version.txt"), @r###"version = "2.0.0-alpha.1""###);
}

#[test]
fn bump_minor_beta() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["Cargo.toml"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            r#"[package]
name = "my-app"
version = "1.2.3"
edition = "2021"
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor-beta").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.3.0-beta.1
    Creating branch: release/1.3.0-beta.1
    Updating version files...
    Committing changes...
    Successfully bumped to 1.3.0-beta.1

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "release/1.3.0-beta.1");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "my-app"
    version = "1.3.0-beta.1"
    edition = "2021"
    "###);
}

#[test]
fn bump_patch_rc() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.0.0",
            &["version"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("version")
        .write_str(r#"version = "1.0.0""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch-rc").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1-rc.1
    Creating branch: release/1.0.1-rc.1
    Updating version files...
    Committing changes...
    Successfully bumped to 1.0.1-rc.1

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("version"), @r###"version = "1.0.1-rc.1""###);
}

#[test]
fn bump_alpha_prerelease() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3-alpha.1",
            &["VERSION"],
            "Bump {version}",
            "rel/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3-alpha.1""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3-alpha.1 to 1.2.3-alpha.2
    Creating branch: rel/1.2.3-alpha.2
    Updating version files...
    Committing changes...
    Successfully bumped to 1.2.3-alpha.2

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "rel/1.2.3-alpha.2");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "1.2.3-alpha.2""###);
}

#[test]
fn bump_beta_prerelease() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "2.0.0-beta.5",
            &["ver.txt"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("ver.txt")
        .write_str(r#"version = "2.0.0-beta.5""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("beta").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.0.0-beta.5 to 2.0.0-beta.6
    Creating branch: release/2.0.0-beta.6
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-beta.6

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("ver.txt"), @r###"version = "2.0.0-beta.6""###);
}

#[test]
fn bump_explicit_version() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["VERSION"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("3.0.0").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 3.0.0
    Creating branch: release/3.0.0
    Updating version files...
    Committing changes...
    Successfully bumped to 3.0.0

    ----- stderr -----
    ");

    assert_eq!(context.git_current_branch(), "release/3.0.0");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "3.0.0""###);
}

#[test]
fn bump_explicit_prerelease_version() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["VERSION"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("2.0.0-beta.1").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0-beta.1
    Creating branch: release/2.0.0-beta.1
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-beta.1

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "2.0.0-beta.1""###);
}

#[test]
fn bump_multiple_files() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "0.1.0",
            &["Cargo.toml", "package.json", "VERSION", "version.py"],
            "Bump to {version}",
            "bump/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            r#"[package]
name = "multi"
version = "0.1.0"
"#,
        )
        .unwrap();

    context
        .root
        .child("package.json")
        .write_str(r#"{"version": "0.1.0"}"#)
        .unwrap();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "0.1.0""#)
        .unwrap();

    context
        .root
        .child("version.py")
        .write_str(r#"__version__ = "0.1.0""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 0.1.0 to 0.2.0
    Creating branch: bump/0.2.0
    Updating version files...
    Committing changes...
    Successfully bumped to 0.2.0

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "multi"
    version = "0.2.0"
    "###);

    insta::assert_snapshot!(context.read_file("package.json"), @r###"{"version": "0.2.0"}"###);

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "0.2.0""###);

    insta::assert_snapshot!(context.read_file("version.py"), @r###"__version__ = "0.2.0""###);
}

#[test]
fn bump_invalid_version_argument() {
    let context = TestContext::new();
    context.minimal_seal_toml("1.2.3").init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("invalid").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to parse version bump argument
      Caused by: invalid version bump: 'invalid'. Expected 'major', 'minor', 'patch', 'alpha', 'beta', 'rc', combinations like 'major-alpha', or a semantic version like '1.2.3'
    ");
}

#[test]
fn bump_prerelease_mismatch() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3-alpha.1",
            &["VERSION"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3-alpha.1""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("beta").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to calculate new version from '1.2.3-alpha.1' with bump 'beta'
      Caused by: invalid version bump: 'Cannot bump beta prerelease on a alpha version'. Expected 'major', 'minor', 'patch', 'alpha', 'beta', 'rc', combinations like 'major-alpha', or a semantic version like '1.2.3'
    ");
}

#[test]
fn bump_prerelease_on_stable() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["VERSION"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to calculate new version from '1.2.3' with bump 'alpha'
      Caused by: invalid version bump: 'Cannot bump prerelease on a stable version'. Expected 'major', 'minor', 'patch', 'alpha', 'beta', 'rc', combinations like 'major-alpha', or a semantic version like '1.2.3'
    ");
}

#[test]
fn bump_missing_version_file() {
    let context = TestContext::new();
    context
        .full_seal_toml(
            "1.2.3",
            &["missing.txt"],
            "Release {version}",
            "release/{version}",
            "v{version}",
        )
        .init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0
    Creating branch: release/2.0.0
    Updating version files...

    ----- stderr -----
    error: Version file not found: [TEMP]/missing.txt
    ");
}

#[test]
fn bump_no_git_repo() {
    let context = TestContext::new();
    context.full_seal_toml(
        "1.2.3",
        &["VERSION"],
        "Release {version}",
        "release/{version}",
        "v{version}",
    );

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0
    Creating branch: release/2.0.0

    ----- stderr -----
    error: Failed to create git branch: fatal: not a git repository (or any parent up to mount point /)
    Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).
    ");
}
