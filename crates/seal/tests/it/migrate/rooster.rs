use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn migrate_rooster_basic() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r"
[tool.rooster]
changelog_ignore_labels = ['internal', 'ci', 'testing']
changelog_contributors = true
",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully migrated rooster config to 'seal.toml'

    Migration warnings:
      - NOTE: You will need to manually add the [release] section with 'current-version'

    See docs/migration.md for more information about unsupported features.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [changelog]
    ignore-labels = [
        "internal",
        "ci",
        "testing",
    ]
    "#);
}

#[test]
fn migrate_rooster_with_section_labels() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r#"
[tool.rooster]
changelog_ignore_labels = ["internal"]

[tool.rooster.section-labels]
"Breaking changes" = ["breaking"]
"Enhancements" = ["enhancement", "compatibility"]
"Bug fixes" = ["bug"]
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully migrated rooster config to 'seal.toml'

    Migration warnings:
      - section-labels/changelog-sections: Custom changelog sections are supported but you need to manually verify the mapping
      - NOTE: You will need to manually add the [release] section with 'current-version'

    See docs/migration.md for more information about unsupported features.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [changelog]
    ignore-labels = ["internal"]

    [changelog.section-labels]
    "Breaking changes" = ["breaking"]
    "Bug fixes" = ["bug"]
    Enhancements = [
        "enhancement",
        "compatibility",
    ]
    "#);
}

#[test]
fn migrate_rooster_with_unsupported_features() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r"
[tool.rooster]
major_labels = []
minor_labels = ['breaking']
default_bump_type = 'pre'
trim_title_prefixes = ['[prefix]']
submodules = ['sub1', 'sub2']
",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully migrated rooster config to 'seal.toml'

    Migration warnings:
      - submodules: Not supported in seal (monorepo members should be configured separately)
      - major-labels/minor-labels: Semantic version bumping based on labels is not yet supported in seal
      - default-bump-type: Not supported in seal (use 'seal bump' with explicit version)
      - trim-title-prefixes: Not supported in seal
      - NOTE: You will need to manually add the [release] section with 'current-version'

    See docs/migration.md for more information about unsupported features.

    ----- stderr -----
    ");
}

#[test]
fn migrate_rooster_with_version_files() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r#"
[tool.rooster]
version_files = [
    "pyproject.toml",
    { path = "Cargo.toml", format = "cargo", field = "package.version" }
]
"#,
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully migrated rooster config to 'seal.toml'

    Migration warnings:
      - current-version set to placeholder '0.0.0' - update this to your actual version

    See docs/migration.md for more information about unsupported features.

    ----- stderr -----
    ");
}

#[test]
fn migrate_rooster_file_already_exists() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r"
[tool.rooster]
changelog_contributors = false
",
        )
        .unwrap();

    let seal_toml = context.root.child("seal.toml");
    seal_toml.write_str("existing content").unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Output file 'seal.toml' already exists. Use --overwrite to overwrite.
    ");
}

#[test]
fn migrate_rooster_force_overwrite() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r"
[tool.rooster]
changelog_contributors = false
",
        )
        .unwrap();

    let seal_toml = context.root.child("seal.toml");
    seal_toml.write_str("existing content").unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml").arg("--overwrite"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully migrated rooster config to 'seal.toml'

    Migration warnings:
      - NOTE: You will need to manually add the [release] section with 'current-version'

    See docs/migration.md for more information about unsupported features.

    ----- stderr -----
    ");

    let contents = fs_err::read_to_string(seal_toml.path()).unwrap();
    insta::assert_snapshot!(contents, @r"
    [changelog]
    include-contributors = false
    ");
}

#[test]
fn migrate_rooster_no_tool_section() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r"
[project]
name = 'test'
",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(pyproject.path()).arg("--output").arg("seal.toml"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to parse rooster config from '[TEMP]/pyproject.toml'
      Caused by: No [tool.rooster] section found
    ");
}

#[test]
fn migrate_rooster_missing_input_file() {
    let context = TestContext::new().with_filtered_missing_file_error();
    let missing_file = context.root.child("missing.toml");

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster").arg("--input").arg(missing_file.path()).arg("--output").arg("seal.toml"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to parse rooster config from '[TEMP]/missing.toml'
      Caused by: Failed to read file: [TEMP]/missing.toml
      Caused by: [OS ERROR 2]
    ");
}

#[test]
fn migrate_rooster_defaults() {
    let context = TestContext::new();
    let pyproject = context.root.child("pyproject.toml");
    pyproject
        .write_str(
            r"
[tool.rooster]
changelog_ignore_labels = ['internal']
",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("migrate").arg("rooster"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully migrated rooster config to 'seal.toml'

    Migration warnings:
      - NOTE: You will need to manually add the [release] section with 'current-version'

    See docs/migration.md for more information about unsupported features.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [changelog]
    ignore-labels = ["internal"]
    "#);
}
