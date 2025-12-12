use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn generate_changelog_no_seal_toml() {
    let context = TestContext::new().with_filtered_missing_file_error();
    context.init_git();

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Failed to read config file [TEMP]/seal.toml: failed to open file `[TEMP]/seal.toml`: [OS ERROR 2]
      Caused by: failed to open file `[TEMP]/seal.toml`: [OS ERROR 2]
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
    error: CHANGELOG.md already exists at `[TEMP]/CHANGELOG.md`. Remove it first if you want to regenerate it.
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

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## 0.2.0

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
    CHANGELOG.md generated successfully.

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("CHANGELOG.md"), @r"
    ## 1.0.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ### Contributors

    - [@alice](https://github.com/alice)

    ## 0.2.0

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

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))

    ## 0.2.0

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

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("changelog").arg("--dry-run").arg("--max-prs").arg("4"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    ## 1.0.0

    ### Documentation

    - Update documentation ([#2](https://github.com/owner/repo/pull/2))


    ----- stderr -----
    ");
}
