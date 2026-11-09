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
"Other changes" = ["__unknown__"]
```

Pull requests with labels matching multiple sections use the first matching section in alphabetical order.
Use the special `__unknown__` label to include pull requests without labels. Pull requests with
labels that are not mapped to a section do not appear in a section.
When no pull requests match a section, Seal writes `No changelog entries for this release.` for the release.

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

## Select Changelog Sections

Use `--only-sections` to include selected configured headings, or `--without-sections` to exclude
headings. Both flags work with `seal bump` and `seal generate changelog`:

```console
seal bump minor --only-sections "Enhancements" --only-sections "Bug fixes" --dry-run
seal generate changelog --without-sections "Other changes" --dry-run
```

Repeat either flag for additional sections. Names must exactly match headings in
`changelog.section-labels`; unknown names are errors. If both flags name the same section,
`--without-sections` takes precedence. These filters cannot be combined with `bump --no-changelog`
and require changelog configuration.

Existing ignored labels and contributors still apply. Section filters select from the usual
section assignments; excluding a section does not move its pull requests into another matching
section. Contributors are credited only for included entries. If no entries remain, the release
says `No changelog entries for this release.`

Omit `--dry-run` to write the same selected entries. Filters change release notes only: they do
not change which pull requests determine `bump auto`'s version.

## Generate Release Metadata

`seal generate release` reads the latest version section and writes JSON containing its title,
body, and pre-release status:

```console
seal generate release
```

This output is intended for release automation.
