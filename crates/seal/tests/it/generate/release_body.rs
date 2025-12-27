use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn generate_release_no_changelog() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("release"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Changelog not found at `[TEMP]/CHANGELOG.md`
    ");
}

#[test]
fn generate_release_empty_changelog() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str("# Changelog\n\nSome text but no version sections.")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("generate").arg("release"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: No version sections found in changelog
    ");
}

#[test]
fn generate_release_stable_version() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 1.0.0\n\n",
            "### Features\n\n",
            "- Added new feature ([#1](https://github.com/owner/repo/pull/1))\n\n",
            "### Contributors\n\n",
            "- [@alice](https://github.com/alice)\n\n",
            "## 0.9.0\n\n",
            "### Bug Fixes\n\n",
            "- Fixed bug ([#2](https://github.com/owner/repo/pull/2))\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}

#[test]
fn generate_release_prerelease_alpha() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "2.0.0-alpha.1"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 2.0.0-alpha.1\n\n",
            "### Breaking Changes\n\n",
            "- API changed ([#10](https://github.com/owner/repo/pull/10))\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}

#[test]
fn generate_release_prerelease_beta() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.5.0-beta.2"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 1.5.0-beta.2\n\n",
            "### Improvements\n\n",
            "- Performance improvements\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}

#[test]
fn generate_release_prerelease_rc() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "3.0.0-rc.1"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 3.0.0-rc.1\n\n",
            "### Bug Fixes\n\n",
            "- Fixed critical bug\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}

#[test]
fn generate_release_with_contributors() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.2.3"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 1.2.3\n\n",
            "### Features\n\n",
            "- Feature A ([#5](url))\n",
            "- Feature B ([#6](url))\n\n",
            "### Bug Fixes\n\n",
            "- Bug fix ([#7](url))\n\n",
            "### Contributors\n\n",
            "- [@alice](https://github.com/alice)\n",
            "- [@bob](https://github.com/bob)\n",
            "- [@charlie](https://github.com/charlie)\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}

#[test]
fn generate_release_custom_changelog_path() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"

[changelog]
changelog-path = "docs/CHANGES.md"
"#,
    );

    context.root.child("docs").create_dir_all().unwrap();

    context
        .root
        .child("docs/CHANGES.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 1.0.0\n\n",
            "### Features\n\n",
            "- Custom path feature\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}

#[test]
fn generate_release_single_version() {
    let context = TestContext::new();
    context.init_git();

    context.seal_toml(
        r#"
[release]
current-version = "1.0.0"
"#,
    );

    context
        .root
        .child("CHANGELOG.md")
        .write_str(concat!(
            "# Changelog\n\n",
            "## 1.0.0\n\n",
            "### Initial Release\n\n",
            "- First version\n"
        ))
        .unwrap();

    let output = context
        .command()
        .arg("generate")
        .arg("release")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let formatted = serde_json::to_string_pretty(&json).unwrap();
    insta::assert_snapshot!(formatted);
}
