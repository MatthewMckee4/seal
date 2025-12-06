use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn validate_project_simple() {
    let context = TestContext::new();
    context.minimal_seal_toml("1.0.0");

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Project validation successful

    ----- stderr -----
    ");
}

#[test]
fn validate_project_with_explicit_path() {
    let context = TestContext::new();
    context.minimal_seal_toml("2.0.0");

    seal_snapshot!(context.command().arg("validate").arg("project").arg("--project").arg(context.temp_dir.path()), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Project validation successful

    ----- stderr -----
    ");
}

#[test]
fn validate_project_short_flag() {
    let context = TestContext::new();
    context.minimal_seal_toml("3.0.0");

    seal_snapshot!(context.command().arg("validate").arg("project").arg("-p").arg(context.temp_dir.path()), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Project validation successful

    ----- stderr -----
    ");
}

#[test]
fn validate_project_not_found() {
    let context = TestContext::new();

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to read config file [TEMP]/seal.toml: No such file or directory (os error 2)
      Caused by: No such file or directory (os error 2)
    ");
}

#[test]
fn validate_project_with_members() {
    let context = TestContext::new();

    let pkg1_dir = context.temp_dir.child("packages/pkg1");
    let pkg2_dir = context.temp_dir.child("packages/pkg2");
    pkg1_dir.create_dir_all().unwrap();
    pkg2_dir.create_dir_all().unwrap();

    context.seal_toml(
        r#"
[members]
pkg1 = "packages/pkg1"
pkg2 = "packages/pkg2"

[release]
current-version = "1.0.0"
"#,
    );

    pkg1_dir
        .child("seal.toml")
        .write_str(
            r#"
[release]
current-version = "0.1.0"
"#,
        )
        .unwrap();

    pkg2_dir
        .child("seal.toml")
        .write_str(
            r#"
[release]
current-version = "0.2.0"
"#,
        )
        .unwrap();

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Project validation successful
    Found 2 workspace member(s)

    ----- stderr -----
    ");
}

#[test]
fn validate_project_member_missing_seal_toml() {
    let context = TestContext::new();

    let pkg_dir = context.temp_dir.child("packages/pkg1");
    pkg_dir.create_dir_all().unwrap();

    context.seal_toml(
        r#"
[members]
pkg1 = "packages/pkg1"

[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Workspace member 'pkg1' is missing seal.toml at path: [TEMP]/packages/pkg1/seal.toml
    ");
}

#[test]
fn validate_project_member_path_not_found() {
    let context = TestContext::new();

    context.seal_toml(
        r#"
[members]
pkg1 = "packages/pkg1"

[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Workspace member 'pkg1' path does not exist: [TEMP]/packages/pkg1
    ");
}

#[test]
fn validate_project_invalid_member_config() {
    let context = TestContext::new();

    let pkg_dir = context.temp_dir.child("packages/pkg1");
    pkg_dir.create_dir_all().unwrap();

    context.seal_toml(
        r#"
[members]
pkg1 = "packages/pkg1"

[release]
current-version = "1.0.0"
"#,
    );

    pkg_dir
        .child("seal.toml")
        .write_str(
            r#"
[release]
current-version = "0.1.0"
commit-message = "No placeholder"
"#,
        )
        .unwrap();

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 4, column 18
      |
    4 | commit-message = "No placeholder"
      |                  ^^^^^^^^^^^^^^^^
    release.commit-message must contain '{version}' placeholder, got: 'No placeholder'
    "#);
}

#[test]
fn validate_project_with_multiple_members() {
    let context = TestContext::new();

    for pkg in ["pkg1", "pkg2", "pkg3", "pkg4"] {
        let pkg_dir = context.temp_dir.child(format!("packages/{pkg}"));
        pkg_dir.create_dir_all().unwrap();
        pkg_dir
            .child("seal.toml")
            .write_str(
                r#"
[release]
current-version = "0.1.0"
"#,
            )
            .unwrap();
    }

    context.seal_toml(
        r#"
[members]
pkg1 = "packages/pkg1"
pkg2 = "packages/pkg2"
pkg3 = "packages/pkg3"
pkg4 = "packages/pkg4"

[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.command().current_dir(&context.temp_dir).arg("validate").arg("project"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Project validation successful
    Found 4 workspace member(s)

    ----- stderr -----
    ");
}
