use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

mod custom_formats;

#[test]
fn bump_no_seal_toml() {
    let context = TestContext::new().with_filtered_missing_file_error();
    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to read config file [TEMP]/seal.toml: failed to open file `[TEMP]/seal.toml`: [OS ERROR 2]
      Caused by: failed to open file `[TEMP]/seal.toml`: [OS ERROR 2]
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_no_release_configuration_in_seal_toml() {
    let context = TestContext::new();

    context.init_git();

    context.root.child("seal.toml").write_str("").unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: No release configuration found in discovered workspace at `[TEMP]/`
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_invalid_bump_name() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("majjor"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to parse version bump argument
      Caused by: invalid version bump: 'majjor'. Expected 'major', 'minor', 'patch', 'alpha', 'beta', 'rc', combinations like 'major-alpha', or a semantic version like '1.2.3'
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_invalid_semver_version() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("1.1.1.1.1"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to parse version bump argument
      Caused by: invalid version bump: '1.1.1.1.1'. Expected 'major', 'minor', 'patch', 'alpha', 'beta', 'rc', combinations like 'major-alpha', or a semantic version like '1.2.3'
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_invalid_current_semver_version() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3.4.5"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Invalid current version
      Caused by: unexpected character '.' after patch version number
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_same_version() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("1.2.3"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Invalid version bump
      Caused by: explicit version '1.2.3' is the same as the current version '1.2.3'
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_prior_version() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("1.2.2"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Invalid version bump
      Caused by: explicit version '1.2.2' is prior to the current version '1.2.3'
    ");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_patch_valid_dry_run() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,2 +1,2 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `seal.toml`

    Dry run complete. No changes made.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_patch_valid_dry_run_single_version_file() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,3 +1,3 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Dry run complete. No changes made.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.3)");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_patch_valid_single_version_file_confirm() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,3 +1,3 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Proceed with these changes? (y/n):
    Updating files...
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_no_changelog_changes_when_no_changelog_option() {
    let context = TestContext::new();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]
"#,
    );

    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("--no-changelog").arg("-v"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
     
     [changelog]
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `seal.toml`

    Proceed with these changes? (y/n):
    ----- stderr -----
    INFO Workspace discovered at "[TEMP]/"
    INFO Warning: No version files configured - only seal.toml will be updated
    INFO Skipping changelog update because `--no-changelog` was provided.

    No changes applied.
    "#);
}

#[test]
fn bump_without_changelog_skips_generation() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,2 +1,2 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `seal.toml`

    Proceed with these changes? (y/n):
    Updating files...
    Successfully bumped to 1.0.1

    ----- stderr -----
    "#);
}

#[test]
fn bump_changelog() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]
ignore-contributors = ["ignored"]

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("-v").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
     
     [changelog]
    ───────────────────────────────────────────────────────────────────────────────
    --- CHANGELOG.md
    +++ CHANGELOG.md
    @@ -0,0 +1,22 @@
    +# Changelog
    +
    +## 1.0.1
    +
    +### Bug Fixes
    +
    +- Fix critical bug in module Y ([#5](https://github.com/owner/repo/pull/5))
    +
    +### Documentation
    +
    +- Update documentation ([#4](https://github.com/owner/repo/pull/4))
    +
    +### New Features
    +
    +- Add new feature X ([#6](https://github.com/owner/repo/pull/6))
    +
    +### Contributors
    +
    +- [@alice](https://github.com/alice)
    +- [@bob](https://github.com/bob)
    +- [@joe](https://github.com/joe)
    +
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `seal.toml`
      - Update `CHANGELOG.md`

    Proceed with these changes? (y/n):
    Updating files...
    Successfully bumped to 1.0.1

    ----- stderr -----
    INFO Workspace discovered at "[TEMP]/"
    INFO Warning: No version files configured - only seal.toml will be updated
    "#);
}

#[test]
fn bump_changelog_different_path() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]
ignore-contributors = ["ignored"]
changelog-path = "CHANGE_LOG.md"

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
     
     [changelog]
    ───────────────────────────────────────────────────────────────────────────────
    --- CHANGE_LOG.md
    +++ CHANGE_LOG.md
    @@ -0,0 +1,22 @@
    +# Changelog
    +
    +## 1.0.1
    +
    +### Bug Fixes
    +
    +- Fix critical bug in module Y ([#5](https://github.com/owner/repo/pull/5))
    +
    +### Documentation
    +
    +- Update documentation ([#4](https://github.com/owner/repo/pull/4))
    +
    +### New Features
    +
    +- Add new feature X ([#6](https://github.com/owner/repo/pull/6))
    +
    +### Contributors
    +
    +- [@alice](https://github.com/alice)
    +- [@bob](https://github.com/bob)
    +- [@joe](https://github.com/joe)
    +
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `seal.toml`
      - Update `CHANGE_LOG.md`

    Proceed with these changes? (y/n):
    Updating files...
    Successfully bumped to 1.0.1

    ----- stderr -----
    "#);
}

#[test]
fn bump_patch_valid_commit() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
commit-message = "Release v{version}"
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
     commit-message = "Release v{version}"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Commands to be executed:
      `git add -A`
      `git commit -m Release v1.2.4`

    Proceed with these changes? (y/n):
    Updating files...
    Executing command: `git add -A`
    Executing command: `git commit -m Release v1.2.4`
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    commit-message = "Release v{version}"
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"main");
    insta::assert_snapshot!(context.git_last_commit_message(), @"Release v1.2.4");
}

#[test]
fn bump_patch_valid_commit_branch() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
     commit-message = "Release v{version}"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Commands to be executed:
      `git checkout -b release/v1.2.4`
      `git add -A`
      `git commit -m Release v1.2.4`

    Proceed with these changes? (y/n):
    Updating files...
    Executing command: `git checkout -b release/v1.2.4`
    Executing command: `git add -A`
    Executing command: `git commit -m Release v1.2.4`
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"release/v1.2.4");
    insta::assert_snapshot!(context.git_last_commit_message(), @"Release v1.2.4");
}

#[test]
fn bump_patch_valid_branch() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
branch-name = "release/v{version}"
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
     branch-name = "release/v{version}"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Commands to be executed:
      `git checkout -b release/v1.2.4`

    Proceed with these changes? (y/n):
    Updating files...
    Executing command: `git checkout -b release/v1.2.4`
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    branch-name = "release/v{version}"
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_patch_valid_commit_branch_push() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = true
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
     commit-message = "Release v{version}"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Commands to be executed:
      `git checkout -b release/v1.2.4`
      `git add -A`
      `git commit -m Release v1.2.4`
      `git push origin release/v1.2.4`

    Proceed with these changes? (y/n):
    Updating files...
    Executing command: `git checkout -b release/v1.2.4`
    Executing command: `git add -A`
    Executing command: `git commit -m Release v1.2.4`
    Executing command: `git push origin release/v1.2.4`
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    push = true
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"release/v1.2.4");
    insta::assert_snapshot!(context.git_last_commit_message(), @"Release v1.2.4");
}

#[test]
fn bump_patch_valid_commit_branch_push_pr() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = true
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
     commit-message = "Release v{version}"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Commands to be executed:
      `git checkout -b release/v1.2.4`
      `git add -A`
      `git commit -m Release v1.2.4`
      `git push origin release/v1.2.4`

    Proceed with these changes? (y/n):
    Updating files...
    Executing command: `git checkout -b release/v1.2.4`
    Executing command: `git add -A`
    Executing command: `git commit -m Release v1.2.4`
    Executing command: `git push origin release/v1.2.4`
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    push = true
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"release/v1.2.4");
    insta::assert_snapshot!(context.git_last_commit_message(), @"Release v1.2.4");
}

#[test]
fn bump_patch_valid_commit_branch_push_pr_no_confirm() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = true
confirm = false
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.4

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.4)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,4 +1,4 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.4"
     version-files = ["README.md"]
     commit-message = "Release v{version}"
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Commands to be executed:
      `git checkout -b release/v1.2.4`
      `git add -A`
      `git commit -m Release v1.2.4`
      `git push origin release/v1.2.4`

    Updating files...
    Executing command: `git checkout -b release/v1.2.4`
    Executing command: `git add -A`
    Executing command: `git commit -m Release v1.2.4`
    Executing command: `git push origin release/v1.2.4`
    Successfully bumped to 1.2.4

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.4)");
    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "1.2.4"
    version-files = ["README.md"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    push = true
    confirm = false
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @"release/v1.2.4");
    insta::assert_snapshot!(context.git_last_commit_message(), @"Release v1.2.4");
}

#[test]
fn bump_alpha_valid_dry_run_single_version_file() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["README.md"]
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.3-alpha.0

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3)
    / No newline at end of file
    +# My Package (1.2.3-alpha.0)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,3 +1,3 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.3-alpha.0"
     version-files = ["README.md"]
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Dry run complete. No changes made.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.3)");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_alpha_already_alpha_valid_dry_run_single_version_file() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3-alpha.0"
version-files = ["README.md"]
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3-alpha.0)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3-alpha.0 to 1.2.3-alpha.1

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3-alpha.0)
    / No newline at end of file
    +# My Package (1.2.3-alpha.1)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,3 +1,3 @@
     [release]
    -current-version = "1.2.3-alpha.0"
    +current-version = "1.2.3-alpha.1"
     version-files = ["README.md"]
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Dry run complete. No changes made.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.3-alpha.0)");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_alpha_base_alpha_valid_dry_run_single_version_file() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3-alpha"
version-files = ["README.md"]
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3-alpha)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3-alpha to 1.2.3-alpha.1

    Preview of changes:
    ───────────────────────────────────────────────────────────────────────────────
    --- README.md
    +++ README.md
    @@ -1 +1 @@
    -# My Package (1.2.3-alpha)
    / No newline at end of file
    +# My Package (1.2.3-alpha.1)
    / No newline at end of file
    ───────────────────────────────────────────────────────────────────────────────
    --- seal.toml
    +++ seal.toml
    @@ -1,3 +1,3 @@
     [release]
    -current-version = "1.2.3-alpha"
    +current-version = "1.2.3-alpha.1"
     version-files = ["README.md"]
    ───────────────────────────────────────────────────────────────────────────────
    Changes to be made:
      - Update `README.md`
      - Update `seal.toml`

    Dry run complete. No changes made.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.3-alpha)");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}

#[test]
fn bump_alpha_invalid_alpha_valid_dry_run_single_version_file() {
    let context = TestContext::new();

    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3-alpha.-1"
version-files = ["README.md"]
"#,
    );

    context
        .root
        .child("README.md")
        .write_str("# My Package (1.2.3-alpha.-1)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--dry-run"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: malformed version: 'Invalid prerelease number in: alpha.-1'. Expected format 'X.Y.Z' where X, Y, and Z are non-negative integers
    ");

    insta::assert_snapshot!(context.read_file("README.md"), @"# My Package (1.2.3-alpha.-1)");

    insta::assert_snapshot!(context.git_current_branch(), @"HEAD");
    insta::assert_snapshot!(context.git_last_commit_message(), @"");
}
