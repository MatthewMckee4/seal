# Changelog

## 0.0.1-alpha.9

### Bug Fixes

- Cover local version bumps without GitHub remotes ([#161](https://github.com/MatthewMckee4/seal/pull/161))
- Describe releases with no changelog entries ([#133](https://github.com/MatthewMckee4/seal/pull/133))
- Restore seal.toml discovery from parent directories ([#156](https://github.com/MatthewMckee4/seal/pull/156))
- Detect every missing version file ([#120](https://github.com/MatthewMckee4/seal/pull/120))
- Continue changelog pagination past unmerged pull requests ([#155](https://github.com/MatthewMckee4/seal/pull/155))

### Bumping

- Add an option to sign release commits ([#182](https://github.com/MatthewMckee4/seal/pull/182))
- Add label driven automatic version bumps ([#163](https://github.com/MatthewMckee4/seal/pull/163))
- Add JSON output for bump dry-run ([#157](https://github.com/MatthewMckee4/seal/pull/157))
- Check Git state before applying a bump ([#98](https://github.com/MatthewMckee4/seal/pull/98))

### CLI

- Add noninteractive bump confirmation flag ([#151](https://github.com/MatthewMckee4/seal/pull/151))

### Changelog

- Filter release notes by changelog section ([#181](https://github.com/MatthewMckee4/seal/pull/181))

### Documentation

- Require command snapshots and focused PR labels ([#162](https://github.com/MatthewMckee4/seal/pull/162))

### New Features

- Add pinned Seal setup action ([#159](https://github.com/MatthewMckee4/seal/pull/159))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.8

### Bug Fixes

- Fix bot contributor links ([#131](https://github.com/MatthewMckee4/seal/pull/131))
- Fix changelog pull request date filtering ([#129](https://github.com/MatthewMckee4/seal/pull/129))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.7

### CLI

- Preview pull request details before confirmation ([#110](https://github.com/MatthewMckee4/seal/pull/110))

### Changelog

- Include unlabeled pull requests in changelogs ([#127](https://github.com/MatthewMckee4/seal/pull/127))

### Configuration

- Update Cargo.lock during releases ([#109](https://github.com/MatthewMckee4/seal/pull/109))

### Documentation

- Improve contributor guides ([#107](https://github.com/MatthewMckee4/seal/pull/107))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.6

### Bug Fixes

- Test git commit hook failure output ([#96](https://github.com/MatthewMckee4/seal/pull/96))

### Bumping

- Create configurable pull requests for release bumps ([#97](https://github.com/MatthewMckee4/seal/pull/97))
- Pre commit failure ([#69](https://github.com/MatthewMckee4/seal/pull/69))
- Support pre commit commands ([#68](https://github.com/MatthewMckee4/seal/pull/68))

### Documentation

- Update repository URLs to karva-dev/seal ([#76](https://github.com/MatthewMckee4/seal/pull/76))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.5

### Bumping

- Update to bump command output ([#55](https://github.com/MatthewMckee4/seal/pull/55))
- Remove create PR ([#54](https://github.com/MatthewMckee4/seal/pull/54))

### Documentation

- Update bump documentation ([#57](https://github.com/MatthewMckee4/seal/pull/57))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.4

### Bumping

- Add more bump tests ([#49](https://github.com/MatthewMckee4/seal/pull/49))

### CLI

- Setup logging ([#48](https://github.com/MatthewMckee4/seal/pull/48))

### Changelog

- Support generating new changelog ([#45](https://github.com/MatthewMckee4/seal/pull/45))

### Documentation

- Improve usage documentation ([#39](https://github.com/MatthewMckee4/seal/pull/39))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.3

### Changelog

- Support ignoring contributors in changelog ([#34](https://github.com/MatthewMckee4/seal/pull/34))
- Changelog support ([#32](https://github.com/MatthewMckee4/seal/pull/32))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.2

### Bug Fixes

- Fixed a bug where `seal bump` release notes were not included in GitHub release.

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.1

### Core Features

- support `seal bump` command ([#7](https://github.com/MatthewMckee4/seal/pull/7))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)
