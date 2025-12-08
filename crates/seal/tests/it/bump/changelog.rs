use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn bump_no_changelog_changes_when_no_changelog_option() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = false
create-pr = false
confirm = false

[release.changelog]
ignore-labels = ["internal", "ci"]
"#,
    );
    context.init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            r#"[package]
name = "test"
version = "1.0.0"
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("--no-changelog"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/Cargo.toml b/[TEMP]/Cargo.toml
    --- a/[TEMP]/Cargo.toml
    +++ b/[TEMP]/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.0.0"
    +version = "1.0.1"

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
     version-files = ["Cargo.toml"]
     commit-message = "Release v{version}"
     branch-name = "release/v{version}"

    Changes to be made:
      - Update `[TEMP]/Cargo.toml`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/v1.0.1`
      `git add -A`
      `git commit -m "Release v1.0.1"`

    Creating branch: release/v1.0.1
    Updating version files...
    Skipping changelog generation as `--no-changelog` was provided.
    Committing changes...
    Successfully bumped to 1.0.1

    ----- stderr -----
    "#);
}

#[test]
fn bump_without_changelog_skips_generation() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
confirm = false
"#,
    );
    context.init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            r#"[package]
name = "test"
version = "1.0.0"
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/Cargo.toml b/[TEMP]/Cargo.toml
    --- a/[TEMP]/Cargo.toml
    +++ b/[TEMP]/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.0.0"
    +version = "1.0.1"

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
     version-files = ["Cargo.toml"]
     confirm = false

    Changes to be made:
      - Update `[TEMP]/Cargo.toml`
      - Update `[TEMP]/seal.toml`

    Note: No branch or commit will be created (branch-name and commit-message not configured)

    Updating version files...
    Skipping changelog generation as no configuration was found.
    Successfully bumped to 1.0.1
    Note: No git branch or commit was created

    ----- stderr -----
    "#);
}
