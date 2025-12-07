use assert_fs::prelude::*;

use crate::{common::TestContext, seal_snapshot};

#[test]
fn bump_with_custom_search_pattern() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "2.5.0"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "version.sh"
search = "export VERSION=\"{version}\""

[[release.version-files]]
path = "config.py"
search = "APP_VERSION = '{version}'"
"#,
        )
        .init_git();

    context
        .root
        .child("version.sh")
        .write_str(concat!(
            "#!/bin/bash\n",
            "export VERSION=\"2.5.0\"\n",
            "export APP_NAME=\"MyApp\"\n"
        ))
        .unwrap();

    context
        .root
        .child("config.py")
        .write_str(concat!(
            "# Configuration\n",
            "APP_VERSION = '2.5.0'\n",
            "DEBUG = False\n"
        ))
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.5.0 to 2.6.0
    Creating branch: release/2.6.0
    Updating version files...
    Committing changes...
    Successfully bumped to 2.6.0

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("version.sh"), @r#"
    #!/bin/bash
    export VERSION="2.6.0"
    export APP_NAME="MyApp"
    "#);

    insta::assert_snapshot!(context.read_file("config.py"), @r"
    # Configuration
    APP_VERSION = '2.6.0'
    DEBUG = False
    ");
}

#[test]
fn bump_with_version_template_major_minor_only() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.2.3"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "VERSION.txt"
search = "Version: {version}"
version-template = "{major}.{minor}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION.txt")
        .write_str("Version: 1.2\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 1.3.0
    Creating branch: release/1.3.0
    Updating version files...

    ----- stderr -----
    error: Search pattern not found in file. Expected: Version: 1.2.3
    ");

    insta::assert_snapshot!(context.read_file("VERSION.txt"), @"Version: 1.2");
}

#[test]
fn bump_with_version_template_with_v_prefix() {
    let context = TestContext::new();
    context
        .seal_toml(
            r##"
[release]
current-version = "3.0.5"
commit-message = "Bump to {version}"
branch-name = "bump/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "version.h"
search = "#define VERSION \"{version}\""
version-template = "v{major}.{minor}.{patch}"
"##,
        )
        .init_git();

    context
        .root
        .child("version.h")
        .write_str(concat!(
            "#ifndef VERSION_H\n",
            "#define VERSION_H\n",
            "#define VERSION \"v3.0.5\"\n",
            "#endif\n"
        ))
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").arg("--no-push").arg("--no-pr"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 3.0.5 to 3.0.6
    Creating branch: bump/3.0.6
    Updating version files...

    ----- stderr -----
    error: Search pattern not found in file. Expected: #define VERSION "3.0.5"
    "#);

    insta::assert_snapshot!(context.read_file("version.h"), @r#"
    #ifndef VERSION_H
    #define VERSION_H
    #define VERSION "v3.0.5"
    #endif
    "#);
}

#[test]
fn bump_with_version_template_prerelease() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "2.0.0-beta.1"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "VERSION"
search = "version={version}"
version-template = "{major}.{minor}.{patch}{extra}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("version=2.0.0-beta.1\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("beta").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.0.0-beta.1 to 2.0.0-beta.2
    Creating branch: release/2.0.0-beta.2
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0-beta.2

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("VERSION"), @"version=2.0.0beta.2");
}

#[test]
fn bump_with_mixed_version_formats() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.5.2"

version-files = [
    "Cargo.toml",
    { path = "version.txt", search = "Version: {version}", version-template = "{major}.{minor}" },
    { path = "full_version.txt", search = "Full: {version}" }
]

commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"
"#,
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(concat!(
            "[package]\n",
            "name = \"myapp\"\n",
            "version = \"1.5.2\"\n"
        ))
        .unwrap();

    context
        .root
        .child("version.txt")
        .write_str("Version: 1.5\n")
        .unwrap();

    context
        .root
        .child("full_version.txt")
        .write_str("Full: 1.5.2\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.5.2 to 1.6.0
    Creating branch: release/1.6.0
    Updating version files...

    ----- stderr -----
    error: Search pattern not found in file. Expected: Version: 1.5.2
    ");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r###"
    [package]
    name = "myapp"
    version = "1.6.0"
    "###);

    insta::assert_snapshot!(context.read_file("version.txt"), @"Version: 1.5");

    insta::assert_snapshot!(context.read_file("full_version.txt"), @"Full: 1.5.2");
}

#[test]
fn bump_with_custom_search_pattern_not_found() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.0.0"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "VERSION"
search = "VERSION={version}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("version=1.0.0\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major").arg("--no-push").arg("--no-pr"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.0.0 to 2.0.0
    Creating branch: release/2.0.0
    Updating version files...

    ----- stderr -----
    error: Search pattern not found in file. Expected: VERSION=1.0.0
    ");
}

#[test]
fn bump_with_version_template_extra_on_stable() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.0.0"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "VERSION"
search = "ver={version}"
version-template = "{major}.{minor}.{patch}{extra}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("ver=1.0.0\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.1.0
    Creating branch: release/1.1.0
    Updating version files...
    Committing changes...
    Successfully bumped to 1.1.0

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("VERSION"), @"ver=1.1.0");
}

#[test]
fn bump_prerelease_to_stable_with_template() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "2.0.0-rc.3"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "VERSION"
search = "version={version}"
version-template = "{major}.{minor}.{patch}{extra}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("version=2.0.0-rc.3\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("2.0.0").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.0.0-rc.3 to 2.0.0
    Creating branch: release/2.0.0
    Updating version files...
    Committing changes...
    Successfully bumped to 2.0.0

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("VERSION"), @"version=2.0.0");
}

#[test]
fn bump_with_template_prerelease_with_hyphen() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "1.0.0-alpha.1"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "version.txt"
search = "APP_VERSION={version}"
version-template = "{major}.{minor}.{patch}-{extra}"
"#,
        )
        .init_git();

    context
        .root
        .child("version.txt")
        .write_str("APP_VERSION=1.0.0-alpha.1\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0-alpha.1 to 1.0.0-alpha.2
    Creating branch: release/1.0.0-alpha.2
    Updating version files...
    Committing changes...
    Successfully bumped to 1.0.0-alpha.2

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("version.txt"), @"APP_VERSION=1.0.0-alpha.2");
}

#[test]
fn bump_with_multiple_occurrences_same_file() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "0.5.0"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

[[release.version-files]]
path = "README.md"
search = "version `{version}`"
"#,
        )
        .init_git();

    context
        .root
        .child("README.md")
        .write_str(concat!(
            "# My Project\n",
            "\n",
            "Current version `0.5.0` is stable.\n",
            "\n",
            "Install version `0.5.0` with npm.\n"
        ))
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor").arg("--no-push").arg("--no-pr"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 0.5.0 to 0.6.0
    Creating branch: release/0.6.0
    Updating version files...
    Committing changes...
    Successfully bumped to 0.6.0

    ----- stderr -----
    ");

    insta::assert_snapshot!(context.read_file("README.md"), @r"
    # My Project

    Current version `0.6.0` is stable.

    Install version `0.6.0` with npm.
    ");
}
