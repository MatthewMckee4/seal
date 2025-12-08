use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn validate_config_valid() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", "README.md"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = false
create-pr = false
confirm = false
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_with_explicit_path() {
    let context = TestContext::new();
    let custom_config = context.root.child("custom.toml");
    custom_config
        .write_str(
            r#"
[release]
current-version = "2.5.0"
version-files = ["VERSION"]
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config").arg("--config-file").arg(custom_config.path()), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/custom.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_minimal() {
    let context = TestContext::new();
    context.minimal_seal_toml("0.1.0");

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_file_not_found() {
    let context = TestContext::new().with_filtered_missing_file_error();

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to read config file [TEMP]/seal.toml: failed to open file `[TEMP]/seal.toml`: [OS ERROR 2]
      Caused by: failed to open file `[TEMP]/seal.toml`: [OS ERROR 2]
    ");
}

#[test]
fn validate_config_explicit_path_not_found() {
    let context = TestContext::new().with_filtered_missing_file_error();
    let missing_config = context.root.child("missing.toml");

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config").arg("--config-file").arg(missing_config.path()), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to read config file [TEMP]/missing.toml: failed to open file `[TEMP]/missing.toml`: [OS ERROR 2]
      Caused by: failed to open file `[TEMP]/missing.toml`: [OS ERROR 2]
    ");
}

#[test]
fn validate_config_invalid_toml() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 1, column 9
      |
    1 | [release
      |         ^
    unclosed table, expected `]`
    ");
}

#[test]
fn validate_config_missing_current_version() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
version-files = ["Cargo.toml"]
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 1, column 1
      |
    1 | [release]
      | ^^^^^^^^^
    missing field `current-version`
    ");
}

#[test]
fn validate_config_empty_commit_message() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
commit-message = ""
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 3, column 18
      |
    3 | commit-message = ""
      |                  ^^
    release.commit-message cannot be empty
    "#);
}

#[test]
fn validate_config_empty_branch_name() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
branch-name = ""
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 3, column 15
      |
    3 | branch-name = ""
      |               ^^
    release.branch-name cannot be empty
    "#);
}

#[test]
fn validate_config_empty_tag_format() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_missing_version_placeholder_in_commit_message() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
commit-message = "Release without placeholder"
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 3, column 18
      |
    3 | commit-message = "Release without placeholder"
      |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    release.commit-message must contain '{version}' placeholder, got: 'Release without placeholder'
    "#);
}

#[test]
fn validate_config_missing_version_placeholder_in_branch_name() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
branch-name = "release-branch"
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 3, column 15
      |
    3 | branch-name = "release-branch"
      |               ^^^^^^^^^^^^^^^^
    release.branch-name must contain '{version}' placeholder, got: 'release-branch'
    "#);
}

#[test]
fn validate_config_missing_version_placeholder_in_tag_format() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_empty_version_files() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = []
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_empty_string_in_version_files() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", ""]
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_unknown_field() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
unknown-field = "value"
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 3, column 1
      |
    3 | unknown-field = "value"
      | ^^^^^^^^^^^^^
    unknown field `unknown-field`, expected one of `current-version`, `version-files`, `commit-message`, `branch-name`, `push`, `create-pr`, `confirm`
    "#);
}

#[test]
fn validate_config_whitespace_only_commit_message() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
commit-message = "   "
"#,
    );

    seal_snapshot!(context.command().arg("validate").arg("config"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: TOML parse error at line 3, column 18
      |
    3 | commit-message = "   "
      |                  ^^^^^
    release.commit-message cannot be empty
    "#);
}

#[test]
fn validate_config_multiple_version_files() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", "pyproject.toml", "package.json", "VERSION"]
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}

#[test]
fn validate_config_custom_patterns() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
commit-message = "bump version to {version}"
branch-name = "releases/{version}"
push = false
create-pr = false
confirm = false
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("validate").arg("config"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Config file `[TEMP]/seal.toml` is valid

    ----- stderr -----
    ");
}
