use crate::{common::TestContext, seal_snapshot};

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

#[test]
fn self_version_short() {
    let context = TestContext::new();

    let filters = context
        .filters()
        .into_iter()
        .chain([(r"\d+\.\d+\.\d+(-alpha\.\d+)?", r"[VERSION]")])
        .collect::<Vec<_>>();

    seal_snapshot!(filters, context.command().arg("self").arg("version").arg("--short"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [VERSION]

    ----- stderr -----
    ");
}

#[test]
fn self_version_json() {
    let context = TestContext::new();

    let filters = context
        .filters()
        .into_iter()
        .chain([(
            r#"version": "\d+\.\d+\.\d+(-(alpha|beta|rc)\.\d+)?(\+\d+)?""#,
            r#"version": "[VERSION]""#,
        )])
        .collect::<Vec<_>>();

    seal_snapshot!(filters, context.command().arg("self").arg("version").arg("--output-format").arg("json"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    {
      "package_name": "seal",
      "version": "[VERSION]"
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
    seal [VERSION]

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
    seal [VERSION]

    ----- stderr -----
    ");
}
