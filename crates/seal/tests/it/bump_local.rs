use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn bump_without_github_remote() {
    let context = TestContext::new();
    context.init_git();
    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["version.txt"]
confirm = false
"#,
    );
    context
        .root
        .child("version.txt")
        .write_str("1.2.3")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().args(["bump", "patch"]), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ────────────────────────────────────────────────────────────────────────────────
    Source: version.txt
    ────────────┬───────────────────────────────────────────────────────────────────
        1       │-1.2.3
              1 │+1.2.4
    ────────────┴───────────────────────────────────────────────────────────────────
    Source: seal.toml
    ────────────┬───────────────────────────────────────────────────────────────────
        1     1 │ [release]
        2       │-current-version = "1.2.3"
              2 │+current-version = "1.2.4"
        3     3 │ version-files = ["version.txt"]
        4     4 │ confirm = false
    ────────────┴───────────────────────────────────────────────────────────────────

    Changes to be made:
      - Update `version.txt`
      - Update `seal.toml`

    Updating files...
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    assert_eq!(context.read_file("version.txt"), "1.2.4");
    assert!(
        context
            .read_file("seal.toml")
            .contains("current-version = \"1.2.4\"")
    );
}

#[test]
fn bump_with_non_github_remote() {
    let context = TestContext::new();
    context.init_git();
    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["version.txt"]
confirm = false
"#,
    );
    context
        .root
        .child("version.txt")
        .write_str("1.2.3")
        .unwrap();
    let status = std::process::Command::new("git")
        .args(["remote", "add", "origin", "https://example.com/owner/repo"])
        .current_dir(context.root.path())
        .status()
        .expect("Failed to add git remote");
    assert!(status.success());

    seal_snapshot!(context.filters(), context.command().args(["bump", "patch"]), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ────────────────────────────────────────────────────────────────────────────────
    Source: version.txt
    ────────────┬───────────────────────────────────────────────────────────────────
        1       │-1.2.3
              1 │+1.2.4
    ────────────┴───────────────────────────────────────────────────────────────────
    Source: seal.toml
    ────────────┬───────────────────────────────────────────────────────────────────
        1     1 │ [release]
        2       │-current-version = "1.2.3"
              2 │+current-version = "1.2.4"
        3     3 │ version-files = ["version.txt"]
        4     4 │ confirm = false
    ────────────┴───────────────────────────────────────────────────────────────────

    Changes to be made:
      - Update `version.txt`
      - Update `seal.toml`

    Updating files...
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    assert_eq!(context.read_file("version.txt"), "1.2.4");
    assert!(
        context
            .read_file("seal.toml")
            .contains("current-version = \"1.2.4\"")
    );
}

#[test]
fn bump_skips_configured_changelog_without_github_remote() {
    let context = TestContext::new();
    context.init_git();
    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["version.txt"]
confirm = false

[changelog]
"#,
    );
    context
        .root
        .child("version.txt")
        .write_str("1.2.3")
        .unwrap();

    seal_snapshot!(
        context.filters(),
        context.command().args(["bump", "patch", "--no-changelog"]),
        @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ────────────────────────────────────────────────────────────────────────────────
    Source: version.txt
    ────────────┬───────────────────────────────────────────────────────────────────
        1       │-1.2.3
              1 │+1.2.4
    ────────────┴───────────────────────────────────────────────────────────────────
    Source: seal.toml
    ────────────┬───────────────────────────────────────────────────────────────────
        1     1 │ [release]
        2       │-current-version = "1.2.3"
              2 │+current-version = "1.2.4"
        3     3 │ version-files = ["version.txt"]
        4     4 │ confirm = false
        5     5 │ 
        6     6 │ [changelog]
    ────────────┴───────────────────────────────────────────────────────────────────

    Changes to be made:
      - Update `version.txt`
      - Update `seal.toml`

    Updating files...
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#
    );

    assert_eq!(context.read_file("version.txt"), "1.2.4");
    assert!(
        context
            .read_file("seal.toml")
            .contains("current-version = \"1.2.4\"")
    );
}

#[test]
#[cfg(not(feature = "integration-test"))]
fn bump_pull_request_rejects_non_github_remote_before_mutation() {
    let context = TestContext::new();
    context.init_git();
    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["version.txt"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = true

[release.pull-request]
"#,
    );
    context
        .root
        .child("version.txt")
        .write_str("1.2.3")
        .unwrap();
    let status = std::process::Command::new("git")
        .args(["remote", "add", "origin", "https://example.com/owner/repo"])
        .current_dir(context.root.path())
        .status()
        .expect("Failed to add git remote");
    assert!(status.success());

    seal_snapshot!(
        context.filters(),
        context.command().args(["bump", "patch"]).write_stdin("y\n"),
        @r#"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ────────────────────────────────────────────────────────────────────────────────
    Source: version.txt
    ────────────┬───────────────────────────────────────────────────────────────────
        1       │-1.2.3
              1 │+1.2.4
    ────────────┴───────────────────────────────────────────────────────────────────
    Source: seal.toml
    ────────────┬───────────────────────────────────────────────────────────────────
        1     1 │ [release]
        2       │-current-version = "1.2.3"
              2 │+current-version = "1.2.4"
        3     3 │ version-files = ["version.txt"]
        4     4 │ commit-message = "Release {version}"
        5     5 │ branch-name = "release/{version}"
        6     6 │ push = true
    ────────────┴───────────────────────────────────────────────────────────────────

    Changes to be made:
      - Update `version.txt`
      - Update `seal.toml`

    Commands to be executed:
      `git checkout -b release/1.2.4`
      `git add -A`
      `git commit -m Release 1.2.4`
      `git push origin release/1.2.4`

    Pull request:
      Title: Release 1.2.4
      Head: release/1.2.4
      Base: main
      Draft: false
      Body: (empty)

    Proceed with these changes? (y/n):

    ----- stderr -----
    error: Invalid GitHub repository URL: https://example.com/owner/repo
    "#
    );
    assert_eq!(context.read_file("version.txt"), "1.2.3");
    assert!(
        context
            .read_file("seal.toml")
            .contains("current-version = \"1.2.3\"")
    );
    assert_eq!(context.git_current_branch(), "HEAD");
}
