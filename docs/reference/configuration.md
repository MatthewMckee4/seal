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

#### [`create-pr`](#release_create-pr)
<span id="create-pr"></span>

Whether to create a pull request for the release changes.

**Default value**: `false`

**Type**: `boolean`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    create-pr = true
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

#### [`tag-format`](#release_tag-format)
<span id="tag-format"></span>

The tag format to use when creating a new tag.

**Default value**: `null`

**Type**: `string`

**Example usage**:

=== "seal.toml"

    ```toml
    [release]
    tag-format = "v{version}"
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
    path = "version.sh"
    search = "export PUBLIC_VERSION=\"{version}\""

    [[release.version-files]]
    path = "Cargo.toml"
    search = "version = \"{version}\""
    ```

---

