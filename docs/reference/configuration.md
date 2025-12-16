## Configuration
### [`members`](#members)

The members of the project.

**Default value**: `{}`

**Type**: `dict`

**Example usage**:

=== "seal.toml"

    ```toml
    [members]
    pkg1 = "packages/pkg1"
    pkg2 = "packages/pkg2"
    ```

---

### `changelog`

#### [`changelog-heading`](#changelog_changelog-heading)
<span id="changelog-heading"></span>

Template for the changelog heading. Must contain {version} placeholder.

**Default value**: `"{version}"`

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [changelog]
    changelog-heading = "{version}"
    ```

---

#### [`changelog-path`](#changelog_changelog-path)
<span id="changelog-path"></span>

Path to the changelog file. Defaults to `CHANGELOG.md`.

**Default value**: `CHANGELOG.md`

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [changelog]
    changelog-path = "CHANGELOG.md"
    ```

---

#### [`ignore-contributors`](#changelog_ignore-contributors)
<span id="ignore-contributors"></span>

Contributors to ignore when generating changelog.

**Default value**: `[]`

**Type**: `list`

**Example usage**:

=== "seal.toml"

    ```toml
    [changelog]
    ignore-contributors = ["dependabot[bot]"]
    ```

---

#### [`ignore-labels`](#changelog_ignore-labels)
<span id="ignore-labels"></span>

Labels to ignore when generating changelog.

**Default value**: `[]`

**Type**: `list`

**Example usage**:

=== "seal.toml"

    ```toml
    [changelog]
    ignore-labels = ["internal", "ci", "testing"]
    ```

---

#### [`include-contributors`](#changelog_include-contributors)
<span id="include-contributors"></span>

Whether to include contributors in the changelog. Defaults to true.

**Default value**: `true`

**Type**: `boolean`

**Example usage**:

=== "seal.toml"

    ```toml
    [changelog]
    include-contributors = true
    ```

---

#### [`section-labels`](#changelog_section-labels)
<span id="section-labels"></span>

Mapping of section names to labels.

**Default value**: `{}`

**Type**: `dict`

**Example usage**:

=== "seal.toml"

    ```toml
    [changelog.section-labels]
    "Breaking changes" = ["breaking"]
    "Enhancements" = ["enhancement", "compatibility"]
    ```

---

### `release`

#### [`branch-name`](#release_branch-name)
<span id="branch-name"></span>

The branch name to use when creating a new release branch.

**Default value**: `null`

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    branch-name = "release-{version}"
    ```

---

#### [`commit-message`](#release_commit-message)
<span id="commit-message"></span>

The commit message to use when committing the release changes.

**Default value**: `null`

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    commit-message = "Release {version}"
    ```

---

#### [`confirm`](#release_confirm)
<span id="confirm"></span>

Whether to confirm the release changes with the user before proceeding.

**Default value**: `true`

**Type**: `boolean`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    confirm = true
    ```

---

#### [`current-version`](#release_current-version)
<span id="current-version"></span>

The current version of the project.

**Required**

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    current-version = "0.1.0"
    ```

---

#### [`push`](#release_push)
<span id="push"></span>

Whether to push the release changes to the remote repository.

**Default value**: `false`

**Type**: `boolean`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    push = false
    ```

---

#### [`version-files`](#release_version-files)
<span id="version-files"></span>

The version files that need to be updated.

**Default value**: `[]`

**Type**: `list`

**Example usage**:

=== "seal.toml"

    ```toml
    [[release.version-files]]
    path = "**/Cargo.toml"
    format = "toml"
    field = "package.version"

    [[release.version-files]]
    path = "version.sh"
    format = "text"

    [[release.version-files]]
    path = "version.sh"
    search = "export FULL_VERSION = '{version}'"

    [[release.version-files]]
    path = "README.md"

    [release]
    version-files = [
        "docs/version.txt"
    ]
    ```

---

