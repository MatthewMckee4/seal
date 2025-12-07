use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn bump_confirm_yes() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
tag-format = "v{version}"
push = false
create-pr = false
confirm = true
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    -------------------

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.0.0"
    +version = "1.0.1"

    Commands to be executed:
      git checkout -b release/v1.0.1
      # Update version files
      # Update seal.toml
      git add -A
      git commit -m "Release v1.0.1"

    Proceed with these changes? (y/n): 
    Creating branch: release/v1.0.1
    Updating version files...
    Updating seal.toml...
    Committing changes...
    Successfully bumped to 1.0.1

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "release/v1.0.1");
    assert_eq!(context.git_last_commit_message(), "Release v1.0.1");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "test"
    version = "1.0.1"
    "###);

    insta::assert_snapshot!(context.read_file("seal.toml"), @r###"

    [release]
    current-version = "1.0.1"
    version-files = ["Cargo.toml"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    tag-format = "v{version}"
    push = false
    create-pr = false
    confirm = true
    "###);
}

#[test]
fn bump_confirm_no() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
tag-format = "v{version}"
push = false
create-pr = false
confirm = true
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("n\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Preview of changes:
    -------------------

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.0.0"
    +version = "1.0.1"

    Commands to be executed:
      git checkout -b release/v1.0.1
      # Update version files
      # Update seal.toml
      git add -A
      git commit -m "Release v1.0.1"

    Proceed with these changes? (y/n): Aborted.

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "main");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "test"
    version = "1.0.0"
    "###);

    insta::assert_snapshot!(context.read_file("seal.toml"), @r###"

    [release]
    current-version = "1.0.0"
    version-files = ["Cargo.toml"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    tag-format = "v{version}"
    push = false
    create-pr = false
    confirm = true
    "###);
}

#[test]
fn bump_no_confirm() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
tag-format = "v{version}"
push = false
create-pr = false
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

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.0.0"
    +version = "1.0.1"

    Commands to be executed:
      git checkout -b release/v1.0.1
      # Update version files
      # Update seal.toml
      git add -A
      git commit -m "Release v1.0.1"

    Creating branch: release/v1.0.1
    Updating version files...
    Updating seal.toml...
    Committing changes...
    Successfully bumped to 1.0.1

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "release/v1.0.1");
    assert_eq!(context.git_last_commit_message(), "Release v1.0.1");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "test"
    version = "1.0.1"
    "###);

    insta::assert_snapshot!(context.read_file("seal.toml"), @r###"

    [release]
    current-version = "1.0.1"
    version-files = ["Cargo.toml"]
    commit-message = "Release v{version}"
    branch-name = "release/v{version}"
    tag-format = "v{version}"
    push = false
    create-pr = false
    confirm = false
    "###);
}
