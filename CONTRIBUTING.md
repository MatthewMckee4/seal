# Contributing

## Finding ways to help

We label issues that would be good for a first time contributor as
[`good first issue`](https://github.com/MatthewMckee4/seal/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
These usually do not require significant experience with code base.

We label issues that we think are a good opportunity for subsequent contributions as
[`help wanted`](https://github.com/MatthewMckee4/seal/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22).
These require varying levels of experience.

## Setup

[Rust](https://rustup.rs/) is required to build and work on the project.

## Testing

For running tests, we recommend [nextest](https://nexte.st/).

If test fail due to mismatch in the reference documentation, run: `cargo run -p seal_dev generate-all`.

### Snapshot testing

we use [insta](https://insta.rs/) for snapshot testing. It's recommended (but not necessary) to use
`cargo-insta` for a better snapshot review experience. See the
[installation guide](https://insta.rs/docs/cli/) for more information.

In tests, you can use `seal_snapshot!` macro to simplify creating snapshots for seal commands. For
example:

```rust
#[test]
fn self_version() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("self").arg("version"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    seal [VERSION]

    ----- stderr -----
    ");
}
```

To run and review a specific snapshot test:

```shell
cargo test --package <package> --test <test> -- <test_name> -- --exact
cargo insta review
```

## Documentation

To prepare and run the documentation locally, run:

```shell
uv run -s scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical serve
```

## Releasing a new version

Funnily enough, we use `seal` to release a new version. To do so, run:

```shell
cargo run bump <version>
```

Then accept the changes.

Then fix any issues there may be.

After merging the pull request, run the
[release workflow](https://github.com/MatthewMckee4/seal/actions/workflows/release.yml) with the version
tag. **Do not include a leading `v`**. The release will automatically be created on GitHub after
everything else publishes.
