# Changelogs

Seal can build changelog sections from merged pull requests in the GitHub repository configured as
the project's Git remote.

## Configure Sections

Add a `[changelog]` section to `seal.toml` and map headings to pull request labels:

```toml
[changelog]
ignore-contributors = ["dependabot[bot]"]
ignore-labels = ["ci", "internal"]
include-contributors = true

[changelog.section-labels]
"Breaking changes" = ["breaking"]
"Enhancements" = ["enhancement"]
"Bug fixes" = ["bug"]
```

Pull requests can appear in more than one section when they have labels from multiple mappings.
Pull requests without a mapped label do not appear in a section.

Seal uses `CHANGELOG.md` by default. Set `changelog-path` to use another file. See the
[configuration reference](../reference/configuration.md) for all filtering and formatting options.

## Update the Changelog During a Bump

When `[changelog]` is configured, `seal bump` fetches pull requests merged since the latest GitHub
release and includes the new version section in its preview:

```console
seal bump minor --dry-run
seal bump minor
```

Pass `--no-changelog` when a particular version bump should update version files only.

Public repositories can use GitHub's unauthenticated API, but setting `GITHUB_TOKEN` or `GH_TOKEN`
avoids the lower anonymous rate limit. Private repositories require a token with access to the
repository.

## Rebuild Changelog History

Generate a changelog from existing GitHub releases and merged pull requests:

```console
seal generate changelog --dry-run
seal generate changelog
```

Seal refuses to replace an existing changelog unless `--overwrite` is passed. Use `--max-prs` to
limit how many recent pull requests Seal fetches before grouping them by release.

## Generate Release Metadata

`seal generate release` reads the latest version section and writes JSON containing its title,
body, and pre-release status:

```console
seal generate release
```

This output is intended for release automation.
