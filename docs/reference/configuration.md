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

<span id="changelog_changelog-heading"></span>
#### [`changelog-heading`](#changelog_changelog-heading)

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

<span id="changelog_changelog-path"></span>
#### [`changelog-path`](#changelog_changelog-path)

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

<span id="changelog_ignore-contributors"></span>
#### [`ignore-contributors`](#changelog_ignore-contributors)

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

<span id="changelog_ignore-labels"></span>
#### [`ignore-labels`](#changelog_ignore-labels)

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

<span id="changelog_include-contributors"></span>
#### [`include-contributors`](#changelog_include-contributors)

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

<span id="changelog_section-labels"></span>
#### [`section-labels`](#changelog_section-labels)

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

<span id="release_branch-name"></span>
#### [`branch-name`](#release_branch-name)

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

<span id="release_commit-message"></span>
#### [`commit-message`](#release_commit-message)

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

<span id="release_confirm"></span>
#### [`confirm`](#release_confirm)

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

<span id="release_current-version"></span>
#### [`current-version`](#release_current-version)

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

<span id="release_on-pre-commit-failure"></span>
#### [`on-pre-commit-failure`](#release_on-pre-commit-failure)

Behavior when a pre-commit command fails.

**Default value**: `abort`

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    on-pre-commit-failure = "abort"  # or "continue"
    ```

---

<span id="release_pre-commit-commands"></span>
#### [`pre-commit-commands`](#release_pre-commit-commands)

Commands to run before committing. These run after `git add -A` and before `git commit`.
A second `git add -A` is run after these commands to stage any changes they make.

**Default value**: `[]`

**Type**: `list`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    pre-commit-commands = ["cargo fmt", "npm run lint:fix"]
    ```

---

<span id="release_push"></span>
#### [`push`](#release_push)

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

<span id="release_version-files"></span>
#### [`version-files`](#release_version-files)

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

