use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn bump_major() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["Cargo.toml", "package.json", "VERSION"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = false
create-pr = false
confirm = false
"#,
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0

    Preview of changes:
    -------------------

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.2.3"
    +version = "2.0.0"

    diff --git a/package.json b/package.json
    --- a/package.json
    +++ b/package.json
    @@ -1 +1 @@
    -{"name": "test", "version": "1.2.3"}
    +{"name": "test", "version": "2.0.0"}

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3"
    +version = "2.0.0"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "2.0.0"
     version-files = ["Cargo.toml", "package.json", "VERSION"]
     commit-message = "Release v{version}"
     branch-name = "release/v{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/Cargo.toml`
      - Update `[TEMP]/package.json`
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/v2.0.0`
      `git add -A`
      `git commit -m "Release v2.0.0"`

    Creating branch: release/v2.0.0
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0

    ----- stderr -----
    "#);

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
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["Cargo.toml"]
commit-message = "bump to {version}"
branch-name = "releases/{version}"
push = false
create-pr = false
confirm = false
"#,
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.3.0

    Preview of changes:
    -------------------

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "test"
    -version = "1.2.3"
    +version = "1.3.0"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.3.0"
     version-files = ["Cargo.toml"]
     commit-message = "bump to {version}"
     branch-name = "releases/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/Cargo.toml`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b releases/1.3.0`
      `git add -A`
      `git commit -m "bump to 1.3.0"`

    Creating branch: releases/1.3.0
    Updating version files...
    Committing changes...
    Successfully bumped to 1.3.0

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "releases/1.3.0");
    assert_eq!(context.git_last_commit_message(), "bump to 1.3.0");

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
        .seal_toml(
            r#"
[release]
current-version = "2.1.5"
version-files = ["VERSION.txt"]
commit-message = "Version {version}"
branch-name = "bump/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION.txt")
        .write_str(r#"version = "2.1.5""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.1.5 to 2.1.6

    Preview of changes:
    -------------------

    diff --git a/VERSION.txt b/VERSION.txt
    --- a/VERSION.txt
    +++ b/VERSION.txt
    @@ -1 +1 @@
    -version = "2.1.5"
    +version = "2.1.6"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "2.1.5"
    +current-version = "2.1.6"
     version-files = ["VERSION.txt"]
     commit-message = "Version {version}"
     branch-name = "bump/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION.txt`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b bump/2.1.6`
      `git add -A`
      `git commit -m "Version 2.1.6"`

    Creating branch: bump/2.1.6
    Updating version files...
    Committing changes...
    Successfully bumped to 2.1.6

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "bump/2.1.6");
    assert_eq!(context.git_last_commit_message(), "Version 2.1.6");

    insta::assert_snapshot!(context.read_file("VERSION.txt"), @r###"version = "2.1.6""###);
}

#[test]
fn bump_major_alpha() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["version.txt"]
commit-message = "Bump {version}"
branch-name = "rel/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("version.txt")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major-alpha"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0-alpha.1

    Preview of changes:
    -------------------

    diff --git a/version.txt b/version.txt
    --- a/version.txt
    +++ b/version.txt
    @@ -1 +1 @@
    -version = "1.2.3"
    +version = "2.0.0-alpha.1"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "2.0.0-alpha.1"
     version-files = ["version.txt"]
     commit-message = "Bump {version}"
     branch-name = "rel/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/version.txt`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b rel/2.0.0-alpha.1`
      `git add -A`
      `git commit -m "Bump 2.0.0-alpha.1"`

    Creating branch: rel/2.0.0-alpha.1
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-alpha.1

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "rel/2.0.0-alpha.1");
    assert_eq!(context.git_last_commit_message(), "Bump 2.0.0-alpha.1");

    insta::assert_snapshot!(context.read_file("version.txt"), @r###"version = "2.0.0-alpha.1""###);
}

#[test]
fn bump_minor_beta() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["Cargo.toml"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor-beta"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.3.0-beta.1

    Preview of changes:
    -------------------

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,4 +1,4 @@
     [package]
     name = "my-app"
    -version = "1.2.3"
    +version = "1.3.0-beta.1"
     edition = "2021"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.3.0-beta.1"
     version-files = ["Cargo.toml"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/Cargo.toml`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.3.0-beta.1`
      `git add -A`
      `git commit -m "Release 1.3.0-beta.1"`

    Creating branch: release/1.3.0-beta.1
    Updating version files...
    Committing changes...
    Successfully bumped to 1.3.0-beta.1

    ----- stderr -----
    "#);

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
        .seal_toml(
            r#"
[release]
current-version = "1.0.0"
version-files = ["version"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("version")
        .write_str(r#"version = "1.0.0""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch-rc"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1-rc.1

    Preview of changes:
    -------------------

    diff --git a/version b/version
    --- a/version
    +++ b/version
    @@ -1 +1 @@
    -version = "1.0.0"
    +version = "1.0.1-rc.1"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1-rc.1"
     version-files = ["version"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/version`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.0.1-rc.1`
      `git add -A`
      `git commit -m "Release 1.0.1-rc.1"`

    Creating branch: release/1.0.1-rc.1
    Updating version files...
    Committing changes...
    Successfully bumped to 1.0.1-rc.1

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("version"), @r#"version = "1.0.1-rc.1""#);
}

#[test]
fn bump_alpha_prerelease() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3-alpha.1"
version-files = ["VERSION"]
commit-message = "Bump {version}"
branch-name = "rel/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3-alpha.1""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3-alpha.1 to 1.2.3-alpha.2

    Preview of changes:
    -------------------

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3-alpha.1"
    +version = "1.2.3-alpha.2"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3-alpha.1"
    +current-version = "1.2.3-alpha.2"
     version-files = ["VERSION"]
     commit-message = "Bump {version}"
     branch-name = "rel/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b rel/1.2.3-alpha.2`
      `git add -A`
      `git commit -m "Bump 1.2.3-alpha.2"`

    Creating branch: rel/1.2.3-alpha.2
    Updating version files...
    Committing changes...
    Successfully bumped to 1.2.3-alpha.2

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "rel/1.2.3-alpha.2");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "1.2.3-alpha.2""###);
}

#[test]
fn bump_beta_prerelease() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "2.0.0-beta.5"
version-files = ["ver.txt"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("ver.txt")
        .write_str(r#"version = "2.0.0-beta.5""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("beta"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.0.0-beta.5 to 2.0.0-beta.6

    Preview of changes:
    -------------------

    diff --git a/ver.txt b/ver.txt
    --- a/ver.txt
    +++ b/ver.txt
    @@ -1 +1 @@
    -version = "2.0.0-beta.5"
    +version = "2.0.0-beta.6"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "2.0.0-beta.5"
    +current-version = "2.0.0-beta.6"
     version-files = ["ver.txt"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/ver.txt`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/2.0.0-beta.6`
      `git add -A`
      `git commit -m "Release 2.0.0-beta.6"`

    Creating branch: release/2.0.0-beta.6
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-beta.6

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("ver.txt"), @r#"version = "2.0.0-beta.6""#);
}

#[test]
fn bump_explicit_version() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("3.0.0"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 3.0.0

    Preview of changes:
    -------------------

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3"
    +version = "3.0.0"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "3.0.0"
     version-files = ["VERSION"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/3.0.0`
      `git add -A`
      `git commit -m "Release 3.0.0"`

    Creating branch: release/3.0.0
    Updating version files...
    Committing changes...
    Successfully bumped to 3.0.0

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "release/3.0.0");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "3.0.0""###);
}

#[test]
fn bump_explicit_prerelease_version() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("2.0.0-beta.1"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0-beta.1

    Preview of changes:
    -------------------

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3"
    +version = "2.0.0-beta.1"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "2.0.0-beta.1"
     version-files = ["VERSION"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/2.0.0-beta.1`
      `git add -A`
      `git commit -m "Release 2.0.0-beta.1"`

    Creating branch: release/2.0.0-beta.1
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-beta.1

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("VERSION"), @r#"version = "2.0.0-beta.1""#);
}

#[test]
fn bump_multiple_files() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "0.1.0"
version-files = ["Cargo.toml", "package.json", "VERSION", "version.py"]
commit-message = "Bump to {version}"
branch-name = "bump/{version}"
push = false
create-pr = false
confirm = false
"#,
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 0.1.0 to 0.2.0

    Preview of changes:
    -------------------

    diff --git a/Cargo.toml b/Cargo.toml
    --- a/Cargo.toml
    +++ b/Cargo.toml
    @@ -1,3 +1,3 @@
     [package]
     name = "multi"
    -version = "0.1.0"
    +version = "0.2.0"

    diff --git a/package.json b/package.json
    --- a/package.json
    +++ b/package.json
    @@ -1 +1 @@
    -{"version": "0.1.0"}
    +{"version": "0.2.0"}

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "0.1.0"
    +version = "0.2.0"

    diff --git a/version.py b/version.py
    --- a/version.py
    +++ b/version.py
    @@ -1 +1 @@
    -__version__ = "0.1.0"
    +__version__ = "0.2.0"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "0.1.0"
    +current-version = "0.2.0"
     version-files = ["Cargo.toml", "package.json", "VERSION", "version.py"]
     commit-message = "Bump to {version}"
     branch-name = "bump/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/Cargo.toml`
      - Update `[TEMP]/package.json`
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/version.py`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b bump/0.2.0`
      `git add -A`
      `git commit -m "Bump to 0.2.0"`

    Creating branch: bump/0.2.0
    Updating version files...
    Committing changes...
    Successfully bumped to 0.2.0

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r#"
    [package]
    name = "multi"
    version = "0.2.0"
    "#);

    insta::assert_snapshot!(context.read_file("package.json"), @r#"{"version": "0.2.0"}"#);

    insta::assert_snapshot!(context.read_file("VERSION"), @r#"version = "0.2.0""#);

    insta::assert_snapshot!(context.read_file("version.py"), @r#"__version__ = "0.2.0""#);
}

#[test]
fn bump_invalid_version_argument() {
    let context = TestContext::new();
    context.minimal_seal_toml("1.2.3").init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("invalid"), @r"
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
        .seal_toml(
            r#"
[release]
current-version = "1.2.3-alpha.1"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3-alpha.1""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("beta"), @r"
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
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3 to 1.2.3-alpha.0

    Preview of changes:
    -------------------

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3"
    +version = "1.2.3-alpha.0"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "1.2.3-alpha.0"
     version-files = ["VERSION"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.2.3-alpha.0`
      `git add -A`
      `git commit -m "Release 1.2.3-alpha.0"`

    Creating branch: release/1.2.3-alpha.0
    Updating version files...
    Committing changes...
    Successfully bumped to 1.2.3-alpha.0

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @r"release/1.2.3-alpha.0");

    insta::assert_snapshot!(context.git_last_commit_message(), @r"Release 1.2.3-alpha.0");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "1.2.3-alpha.0""###);

    context.merge_current_branch_and_checkout_main();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.2.3-alpha.0 to 1.2.3-alpha.1

    Preview of changes:
    -------------------

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3-alpha.0"
    +version = "1.2.3-alpha.1"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3-alpha.0"
    +current-version = "1.2.3-alpha.1"
     version-files = ["VERSION"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.2.3-alpha.1`
      `git add -A`
      `git commit -m "Release 1.2.3-alpha.1"`

    Creating branch: release/1.2.3-alpha.1
    Updating version files...
    Committing changes...
    Successfully bumped to 1.2.3-alpha.1

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.git_current_branch(), @r"release/1.2.3-alpha.1");

    insta::assert_snapshot!(context.git_last_commit_message(), @r"Release 1.2.3-alpha.1");

    insta::assert_snapshot!(context.read_file("VERSION"), @r#"version = "1.2.3-alpha.1""#);
}

#[test]
fn bump_missing_version_file() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["missing.txt"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0

    Preview of changes:
    -------------------

    ----- stderr -----
    error: No files found for path or glob `missing.txt`
    ");
}

#[test]
fn bump_no_git_repo() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
    );

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 2.0.0

    Preview of changes:
    -------------------

    diff --git a/VERSION b/VERSION
    --- a/VERSION
    +++ b/VERSION
    @@ -1 +1 @@
    -version = "1.2.3"
    +version = "2.0.0"

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.2.3"
    +current-version = "2.0.0"
     version-files = ["VERSION"]
     commit-message = "Release {version}"
     branch-name = "release/{version}"
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/2.0.0`
      `git add -A`
      `git commit -m "Release 2.0.0"`

    Creating branch: release/2.0.0

    ----- stderr -----
    error: Failed to create git branch
    "#);
}

#[test]
fn bump_no_version_files() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.0.0"
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.0.1

    Warning: No version files configured - only seal.toml will be updated

    Preview of changes:
    -------------------

    diff --git a/seal.toml b/seal.toml
    --- a/seal.toml
    +++ b/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.0.1"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     push = false
    Skipping changelog update because no `[changelog]` section was found in the configuration.

    Changes to be made:
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.0.1`
      `git add -A`
      `git commit -m "Release 1.0.1"`

    Creating branch: release/1.0.1
    Updating version files...
    Committing changes...
    Successfully bumped to 1.0.1

    ----- stderr -----
    "#);

    assert_eq!(context.git_current_branch(), "release/1.0.1");
    assert_eq!(context.git_last_commit_message(), "Release 1.0.1");

    insta::assert_snapshot!(context.read_file("seal.toml"), @r###"

    [release]
    current-version = "1.0.1"
    commit-message = "Release {version}"
    branch-name = "release/{version}"
    push = false
    create-pr = false
    confirm = false
    "###);
}

#[test]
/// User tries to bump to a version before the current one.
fn bump_explicit_prior_version() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("1.0.0"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to calculate new version from '1.2.3' with bump '1.0.0'
      Caused by: explicit version '1.0.0' is prior to the current version '1.2.3'
    ");

    assert_eq!(context.git_current_branch(), "main");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "1.2.3""###);
}

#[test]
/// User tries to bump to a version equal to the current one.
fn bump_explicit_equal_version() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
version-files = ["VERSION"]
commit-message = "Release {version}"
branch-name = "release/{version}"
push = false
create-pr = false
confirm = false
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str(r#"version = "1.2.3""#)
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("1.2.3"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to calculate new version from '1.2.3' with bump '1.2.3'
      Caused by: explicit version '1.2.3' is the same as the current version '1.2.3'
    ");

    assert_eq!(context.git_current_branch(), "main");

    insta::assert_snapshot!(context.read_file("VERSION"), @r###"version = "1.2.3""###);
}
