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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.5.0 to 2.6.0

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/version.sh b/[TEMP]/version.sh
    --- a/[TEMP]/version.sh
    +++ b/[TEMP]/version.sh
    @@ -1,3 +1,3 @@
     #!/bin/bash
    -export VERSION="2.5.0"
    +export VERSION="2.6.0"
     export APP_NAME="MyApp"

    diff --git a/[TEMP]/config.py b/[TEMP]/config.py
    --- a/[TEMP]/config.py
    +++ b/[TEMP]/config.py
    @@ -1,3 +1,3 @@
     # Configuration
    -APP_VERSION = '2.5.0'
    +APP_VERSION = '2.6.0'
     DEBUG = False

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "2.5.0"
    +current-version = "2.6.0"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     

    Changes to be made:
      - Update `[TEMP]/version.sh`
      - Update `[TEMP]/config.py`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/2.6.0`
      `git add -A`
      `git commit -m "Release 2.6.0"`

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("version.sh"), @r#"
    #!/bin/bash
    export VERSION="2.5.0"
    export APP_NAME="MyApp"
    "#);

    insta::assert_snapshot!(context.read_file("config.py"), @r"
    # Configuration
    APP_VERSION = '2.5.0'
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

[[release.version-files]]
path = "VERSION.txt"
search = "Version: {version}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION.txt")
        .write_str("Version: 1.2\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.2.3 to 1.3.0

    Preview of changes:
    -------------------

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
            r##"[release]
current-version = "3.0.5"
commit-message = "Bump to {version}"
branch-name = "bump/{version}"

[[release.version-files]]
path = "version.h"
search = "#define VERSION \"{version}\""
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 3.0.5 to 3.0.6

    Preview of changes:
    -------------------

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

[[release.version-files]]
path = "VERSION"
search = "version={version}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("version=2.0.0-beta.1\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("beta"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.0.0-beta.1 to 2.0.0-beta.2

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/VERSION b/[TEMP]/VERSION
    --- a/[TEMP]/VERSION
    +++ b/[TEMP]/VERSION
    @@ -1 +1 @@
    -version=2.0.0-beta.1
    +version=2.0.0-beta.2

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "2.0.0-beta.1"
    +current-version = "2.0.0-beta.2"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/2.0.0-beta.2`
      `git add -A`
      `git commit -m "Release 2.0.0-beta.2"`

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("VERSION"), @"version=2.0.0-beta.1");
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
    { path = "version.txt", search = "Version: {version}" },
    { path = "full_version.txt", search = "Full: {version}" }
]

commit-message = "Release {version}"
branch-name = "release/{version}"
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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.5.2 to 1.6.0

    Preview of changes:
    -------------------

    ----- stderr -----
    error: Search pattern not found in file. Expected: Version: 1.5.2
    ");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r#"
    [package]
    name = "myapp"
    version = "1.5.2"
    "#);

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

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("major"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 1.0.0 to 2.0.0

    Preview of changes:
    -------------------

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

[[release.version-files]]
path = "VERSION"
search = "ver={version}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("ver=1.0.0\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0 to 1.1.0

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/VERSION b/[TEMP]/VERSION
    --- a/[TEMP]/VERSION
    +++ b/[TEMP]/VERSION
    @@ -1 +1 @@
    -ver=1.0.0
    +ver=1.1.0

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.0.0"
    +current-version = "1.1.0"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.1.0`
      `git add -A`
      `git commit -m "Release 1.1.0"`

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("VERSION"), @"ver=1.0.0");
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

[[release.version-files]]
path = "VERSION"
search = "version={version}"
"#,
        )
        .init_git();

    context
        .root
        .child("VERSION")
        .write_str("version=2.0.0-rc.3\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("2.0.0"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 2.0.0-rc.3 to 2.0.0

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/VERSION b/[TEMP]/VERSION
    --- a/[TEMP]/VERSION
    +++ b/[TEMP]/VERSION
    @@ -1 +1 @@
    -version=2.0.0-rc.3
    +version=2.0.0

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "2.0.0-rc.3"
    +current-version = "2.0.0"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     

    Changes to be made:
      - Update `[TEMP]/VERSION`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/2.0.0`
      `git add -A`
      `git commit -m "Release 2.0.0"`

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("VERSION"), @"version=2.0.0-rc.3");
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

[[release.version-files]]
path = "version.txt"
search = "APP_VERSION={version}"
"#,
        )
        .init_git();

    context
        .root
        .child("version.txt")
        .write_str("APP_VERSION=1.0.0-alpha.1\n")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("alpha"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 1.0.0-alpha.1 to 1.0.0-alpha.2

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/version.txt b/[TEMP]/version.txt
    --- a/[TEMP]/version.txt
    +++ b/[TEMP]/version.txt
    @@ -1 +1 @@
    -APP_VERSION=1.0.0-alpha.1
    +APP_VERSION=1.0.0-alpha.2

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "1.0.0-alpha.1"
    +current-version = "1.0.0-alpha.2"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     

    Changes to be made:
      - Update `[TEMP]/version.txt`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/1.0.0-alpha.2`
      `git add -A`
      `git commit -m "Release 1.0.0-alpha.2"`

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("version.txt"), @"APP_VERSION=1.0.0-alpha.1");
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
            "Current version `0.5.0` is stable.\n",
            "Install version `0.5.0` with npm.\n"
        ))
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("minor"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 0.5.0 to 0.6.0

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/README.md b/[TEMP]/README.md
    --- a/[TEMP]/README.md
    +++ b/[TEMP]/README.md
    @@ -1,3 +1,3 @@
     # My Project
    -Current version `0.5.0` is stable.
    -Install version `0.5.0` with npm.
    +Current version `0.6.0` is stable.
    +Install version `0.6.0` with npm.

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "0.5.0"
    +current-version = "0.6.0"
     commit-message = "Release {version}"
     branch-name = "release/{version}"
     

    Changes to be made:
      - Update `[TEMP]/README.md`
      - Update `[TEMP]/seal.toml`

    Commands to be executed:
      `git checkout -b release/0.6.0`
      `git add -A`
      `git commit -m "Release 0.6.0"`

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @r"
    # My Project
    Current version `0.5.0` is stable.
    Install version `0.5.0` with npm.
    ");
}

#[test]
fn bump_version_in_nested_string() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "0.0.1"

[[release.version-files]]
path = "README.md"
"#,
        )
        .init_git();

    context
        .root
        .child("README.md")
        .write_str("# Tool(0.0.1)")
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 0.0.1 to 0.0.2

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/README.md b/[TEMP]/README.md
    --- a/[TEMP]/README.md
    +++ b/[TEMP]/README.md
    @@ -1 +1 @@
    -# Tool(0.0.1)
    +# Tool(0.0.2)

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "0.0.1"
    +current-version = "0.0.2"
     
     [[release.version-files]]
     path = "README.md"

    Changes to be made:
      - Update `[TEMP]/README.md`
      - Update `[TEMP]/seal.toml`

    Note: No branch or commit will be created (branch-name and commit-message not configured)

    Proceed with these changes? (y/n):
    No changes applied.

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("README.md"), @"# Tool(0.0.1)");
}

#[test]
fn bump_version_with_glob_path() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "0.0.1"

[[release.version-files]]
path = "**/Cargo.toml"
"#,
        )
        .init_git();

    context
        .root
        .child("crates/foo/Cargo.toml")
        .write_str(
            "[package]
name = \"foo\"
version = \"0.0.1\"
        ",
        )
        .unwrap();

    context
        .root
        .child("crates/bar/Cargo.toml")
        .write_str(
            "[package]
name = \"bar\"
version = \"0.0.1\"
        ",
        )
        .unwrap();

    context
        .root
        .child("crates/baz/Cargo.toml")
        .write_str(
            "[package]
name = \"baz\"
version = \"0.0.1\"
        ",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Bumping version from 0.0.1 to 0.0.2

    Preview of changes:
    -------------------

    diff --git a/[TEMP]/crates/bar/Cargo.toml b/[TEMP]/crates/bar/Cargo.toml
    --- a/[TEMP]/crates/bar/Cargo.toml
    +++ b/[TEMP]/crates/bar/Cargo.toml
    @@ -1,4 +1,4 @@
     [package]
     name = "bar"
    -version = "0.0.1"
    +version = "0.0.2"
             

    diff --git a/[TEMP]/crates/baz/Cargo.toml b/[TEMP]/crates/baz/Cargo.toml
    --- a/[TEMP]/crates/baz/Cargo.toml
    +++ b/[TEMP]/crates/baz/Cargo.toml
    @@ -1,4 +1,4 @@
     [package]
     name = "baz"
    -version = "0.0.1"
    +version = "0.0.2"
             

    diff --git a/[TEMP]/crates/foo/Cargo.toml b/[TEMP]/crates/foo/Cargo.toml
    --- a/[TEMP]/crates/foo/Cargo.toml
    +++ b/[TEMP]/crates/foo/Cargo.toml
    @@ -1,4 +1,4 @@
     [package]
     name = "foo"
    -version = "0.0.1"
    +version = "0.0.2"
             

    diff --git a/[TEMP]/seal.toml b/[TEMP]/seal.toml
    --- a/[TEMP]/seal.toml
    +++ b/[TEMP]/seal.toml
    @@ -1,5 +1,5 @@
     [release]
    -current-version = "0.0.1"
    +current-version = "0.0.2"
     
     [[release.version-files]]
     path = "**/Cargo.toml"

    Changes to be made:
      - Update `[TEMP]/crates/bar/Cargo.toml`
      - Update `[TEMP]/crates/baz/Cargo.toml`
      - Update `[TEMP]/crates/foo/Cargo.toml`
      - Update `[TEMP]/seal.toml`

    Note: No branch or commit will be created (branch-name and commit-message not configured)

    Proceed with these changes? (y/n):
    Updating version files...
    Skipping changelog generation as no configuration was found.
    Successfully bumped to 0.0.2
    Note: No git branch or commit was created

    ----- stderr -----
    "#);

    insta::assert_snapshot!(context.read_file("crates/foo/Cargo.toml"), @r#"
    [package]
    name = "foo"
    version = "0.0.2"
    "#);

    insta::assert_snapshot!(context.read_file("crates/bar/Cargo.toml"), @r#"
    [package]
    name = "bar"
    version = "0.0.2"
    "#);

    insta::assert_snapshot!(context.read_file("crates/baz/Cargo.toml"), @r#"
    [package]
    name = "baz"
    version = "0.0.2"
    "#);

    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "0.0.2"

    [[release.version-files]]
    path = "**/Cargo.toml"
    "#);
}

#[test]
fn bump_version_cargo_toml_invalid_field_name() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "0.0.1"

[[release.version-files]]
path = "Cargo.toml"
format = "toml"
field = "package.vversion"
"#,
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            "[package]
name = \"foo\"
version = \"0.0.1\"
        ",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 0.0.1 to 0.0.2

    Preview of changes:
    -------------------

    ----- stderr -----
    error: Expected `package` to refer to a TOML table
    ");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r#"
    [package]
    name = "foo"
    version = "0.0.1"
    "#);

    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "0.0.1"

    [[release.version-files]]
    path = "Cargo.toml"
    format = "toml"
    field = "package.vversion"
    "#);
}

#[test]
fn bump_version_cargo_toml_invalid_version_format() {
    let context = TestContext::new();
    context
        .seal_toml(
            r#"
[release]
current-version = "0.0.1"

[[release.version-files]]
path = "Cargo.toml"
format = "toml"
field = "package.version.version"
"#,
        )
        .init_git();

    context
        .root
        .child("Cargo.toml")
        .write_str(
            "[package.version]
vversion = \"0.0.1\"
        ",
        )
        .unwrap();

    seal_snapshot!(context.filters(), context.command().arg("bump").arg("patch").write_stdin("y\n"), @r"
    success: false
    exit_code: 2
    ----- stdout -----
    Bumping version from 0.0.1 to 0.0.2

    Preview of changes:
    -------------------

    ----- stderr -----
    error: Expected `package` to refer to a TOML table
    ");

    insta::assert_snapshot!(context.read_file("Cargo.toml"), @r#"
    [package.version]
    vversion = "0.0.1"
    "#);

    insta::assert_snapshot!(context.read_file("seal.toml"), @r#"
    [release]
    current-version = "0.0.1"

    [[release.version-files]]
    path = "Cargo.toml"
    format = "toml"
    field = "package.version.version"
    "#);
}
