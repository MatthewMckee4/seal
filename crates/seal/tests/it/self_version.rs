use crate::{common::TestContext, seal_snapshot};

#[test]
fn self_version() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("self").arg("version"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    seal 0.0.0
    ----- stderr -----
    ");
}

#[test]
fn self_version_short() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("self").arg("version").arg("--short"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    0.0.0
    ----- stderr -----
    ");
}

#[test]
fn self_version_json() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("self").arg("version").arg("--output-format").arg("json"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    {
      "package_name": "seal",
      "version": "0.0.0"
    }
    ----- stderr -----
    "#);
}

#[test]
fn version_flag() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("--version"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    seal 0.0.0
    ----- stderr -----
    ");
}

#[test]
fn version_short_flag() {
    let context = TestContext::new();

    seal_snapshot!(context.command().arg("-V"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    seal 0.0.0
    ----- stderr -----
    ");
}
