use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn generate_changelog_no_seal_toml() {
    let context = TestContext::new().with_filtered_missing_file_error();
    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog"), @"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Could not find seal.toml. Searched directories:
      - [TEMP]/
    ");
}

#[test]
fn generate_changelog_no_changelog_config() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: No changelog configuration found in discovered workspace at `[TEMP]/`
    ");
}

#[test]
fn generate_changelog_file_already_exists() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal"]
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str("# Existing changelog")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Changelog already exists at `CHANGELOG.md`. Remove it first if you want to regenerate it.
    ");
}

#[test]
fn generate_changelog_basic() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog").arg("--dry-run"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    ## 1.0.0

    ### Documentation

    - Update documentation ([#3](https://github.com/owner/repo/pull/3))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## 0.2.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## Release 2025-11-05

    ### Documentation

    - Update documentation ([#1](https://github.com/owner/repo/pull/1))

    ### Contributors

    - [@alice](https://github.com/alice)


    ----- stderr -----
    ");

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Changelog generated successfully at `CHANGELOG.md`.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("CHANGELOG.md"), @r"
    ## 1.0.0

    ### Documentation

    - Update documentation ([#3](https://github.com/owner/repo/pull/3))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## 0.2.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## Release 2025-11-05

    ### Documentation

    - Update documentation ([#1](https://github.com/owner/repo/pull/1))

    ### Contributors

    - [@alice](https://github.com/alice)
    ");
}

#[test]
fn generate_changelog_without_contributors() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]
include-contributors = false

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog").arg("--dry-run"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    ## 1.0.0

    ### Documentation

    - Update documentation ([#3](https://github.com/owner/repo/pull/3))

    ## 0.2.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ## Release 2025-11-05

    ### Documentation

    - Update documentation ([#1](https://github.com/owner/repo/pull/1))


    ----- stderr -----
    ");
}

#[test]
fn generate_changelog_max_prs() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]
include-contributors = false

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog").arg("--dry-run").arg("--max-prs").arg("5"), @"
    success: true
    exit_code: 0
    ----- stdout -----
    ## 1.0.0

    ### Documentation

    - Update documentation ([#3](https://github.com/owner/repo/pull/3))

    ## 0.2.0

    No changelog entries for this release.

    ## Release 2025-11-05

    No changelog entries for this release.


    ----- stderr -----
    ");
}

#[test]
fn generate_changelog_different_changelog_path() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]
changelog-path = "CHANGE_LOG.md"

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Changelog generated successfully at `CHANGE_LOG.md`.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("CHANGE_LOG.md"), @r"
    ## 1.0.0

    ### Documentation

    - Update documentation ([#3](https://github.com/owner/repo/pull/3))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## 0.2.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## Release 2025-11-05

    ### Documentation

    - Update documentation ([#1](https://github.com/owner/repo/pull/1))

    ### Contributors

    - [@alice](https://github.com/alice)
    ");
}

#[test]
fn generate_changelog_file_already_exists_with_overwrite() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
ignore-labels = ["internal", "ci"]

[changelog.section-labels]
"Bug Fixes" = ["bug"]
"New Features" = ["enhancement", "feature"]
"Documentation" = ["documentation"]
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str("# Existing changelog")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog").arg("--overwrite"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Changelog generated successfully at `CHANGELOG.md`.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("CHANGELOG.md"), @r"
    ## 1.0.0

    ### Documentation

    - Update documentation ([#3](https://github.com/owner/repo/pull/3))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## 0.2.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## Release 2025-11-05

    ### Documentation

    - Update documentation ([#1](https://github.com/owner/repo/pull/1))

    ### Contributors

    - [@alice](https://github.com/alice)
    ");
}

#[test]
fn generate_changelog_from_subdirectory_uses_parent_config_path() {
    let context = TestContext::new();
    context.init_git();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
changelog-path = "docs/CHANGELOG.md"
"#,
    );
    context
        .root
        .child("docs")
        .create_dir_all()
        .expect("Failed to create docs directory");
    context
        .root
        .child("docs/CHANGELOG.md")
        .write_str("Existing changelog")
        .expect("Failed to write changelog");
    let subdirectory = context.root.child("crates/seal");
    subdirectory
        .create_dir_all()
        .expect("Failed to create subdirectory");
    let mut command = context.command();
    command.current_dir(subdirectory.path());

    seal_snapshot!(context.filters(), command.arg("generate").arg("changelog"), @"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Changelog already exists at `docs/CHANGELOG.md`. Remove it first if you want to regenerate it.
    ");
    assert_eq!(context.read_file("docs/CHANGELOG.md"), "Existing changelog");
    assert!(!subdirectory.child("docs/CHANGELOG.md").path().exists());
}

#[test]
fn generate_changelog_with_empty_release_ranges() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
[changelog]
"#,
    );
    context.init_git();
    seal_snapshot!(context.filters(), context.command().args([
        "generate", "changelog", "--dry-run", "--max-prs", "1",
    ]), @"
    success: true
    exit_code: 0
    ----- stdout -----
    ## 1.0.0

    No changelog entries for this release.

    ## 0.2.0

    No changelog entries for this release.

    ## Release 2025-11-05

    No changelog entries for this release.


    ----- stderr -----
    ");
}

#[test]
fn generate_changelog_section_exclusions_win_for_preview_and_write() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
[changelog.section-labels]
Documentation = ["documentation"]
"#,
    );
    context.init_git();
    let filters = [
        "--only-sections",
        "Documentation",
        "--without-sections",
        "Documentation",
    ];
    let preview = seal_snapshot!(context.filters(), context.command().args(["generate", "changelog", "--dry-run"]).args(filters), @"
    success: true
    exit_code: 0
    ----- stdout -----
    ## 1.0.0

    No changelog entries for this release.

    ## 0.2.0

    No changelog entries for this release.

    ## Release 2025-11-05

    No changelog entries for this release.


    ----- stderr -----
    ");
    assert!(!context.root.child("CHANGELOG.md").path().exists());
    seal_snapshot!(context.filters(), context.command().args(["generate", "changelog"]).args(filters), @"
    success: true
    exit_code: 0
    ----- stdout -----
    Changelog generated successfully at `CHANGELOG.md`.

    ----- stderr -----
    ");
    assert_eq!(
        context.read_file("CHANGELOG.md"),
        String::from_utf8(preview.stdout).expect("Changelog must be UTF-8")
    );
}

#[test]
fn generate_changelog_rejects_unknown_section_before_write() {
    let context = TestContext::new();
    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
[changelog.section-labels]
Documentation = ["documentation"]
"#,
    );
    seal_snapshot!(context.filters(), context.command().args(["generate", "changelog", "--without-sections", "Missing"]), @"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Unknown changelog section `Missing`. Available sections: Documentation
    ");
    assert!(!context.root.child("CHANGELOG.md").path().exists());
}
