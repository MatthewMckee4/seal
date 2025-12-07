//! Generate a Markdown-compatible listing of configuration options for `seal.toml`.

use std::path::PathBuf;

use anyhow::bail;

use crate::{Mode, REGENERATE_ALL_COMMAND, ROOT_DIR};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Write the generated table to stdout (rather than to `docs/configuration.md`).
    #[arg(long, default_value_t, value_enum)]
    pub(crate) mode: Mode,
}

pub(crate) fn main(args: &Args) -> anyhow::Result<()> {
    let mut output = String::new();
    let file_name = "docs/configuration.md";
    let markdown_path = PathBuf::from(ROOT_DIR).join(file_name);

    if !markdown_path.exists() {
        std::fs::File::create(&markdown_path)?;
    }

    output.push_str(
        "<!-- WARNING: This file is auto-generated (cargo run -p seal_dev generate-all). Update the doc comments in 'crates/seal_project/src/config.rs' if you want to change anything here. -->\n\n",
    );

    output.push_str("# Configuration\n\n");
    output.push_str("Seal configuration is defined in a `seal.toml` file in the root of your project.\n\n");

    generate_release_config(&mut output);

    match args.mode {
        Mode::DryRun => {
            println!("{output}");
        }
        Mode::Check => {
            let current = std::fs::read_to_string(&markdown_path)?;
            if output == current {
                println!("Up-to-date: {file_name}",);
            } else {
                bail!("{file_name} changed, please run `{REGENERATE_ALL_COMMAND}`");
            }
        }
        Mode::Write => {
            let current = std::fs::read_to_string(&markdown_path)?;
            if current == output {
                println!("Up-to-date: {file_name}",);
            } else {
                println!("Updating: {file_name}",);
                std::fs::write(markdown_path, output.as_bytes())?;
            }
        }
    }

    Ok(())
}

fn generate_release_config(output: &mut String) {
    output.push_str("## `[release]`\n\n");
    output.push_str("Release management configuration.\n\n");

    output.push_str("### `current-version`\n\n");
    output.push_str("**Required**\n\n");
    output.push_str("**Type**: `string`\n\n");
    output.push_str("The current version of the project.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\ncurrent-version = \"1.2.3\"\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `version-files`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `array` of strings or objects\n\n");
    output.push_str("List of files to update with the new version. Can be simple file paths or detailed configurations with search patterns.\n\n");
    output.push_str("**Example (simple)**:\n\n");
    output.push_str("```toml\n[release]\nversion-files = [\"Cargo.toml\", \"package.json\"]\n```\n\n");
    output.push_str("**Example (detailed)**:\n\n");
    output.push_str("```toml\n[[release.version-files]]\npath = \"version.sh\"\nsearch = 'export VERSION=\"{version}\"'\n\n[[release.version-files]]\npath = \"Cargo.toml\"\nsearch = 'version = \"{version}\"'\nversion-template = \"{version}\"\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `commit-message`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `string`\n\n");
    output.push_str("Commit message template for version bump commits. Must contain `{version}` placeholder.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\ncommit-message = \"chore: release v{version}\"\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `branch-name`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `string`\n\n");
    output.push_str("Branch name template for release branches. Must contain `{version}` placeholder. Required if `push` or `create-pr` is true.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\nbranch-name = \"release-{version}\"\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `tag-format`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `string`\n\n");
    output.push_str("Git tag format template. Must contain `{version}` placeholder.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\ntag-format = \"v{version}\"\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `push`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `boolean`\n\n");
    output.push_str("**Default**: `false`\n\n");
    output.push_str("Whether to push the release branch to the remote repository. Requires `branch-name` to be set.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\npush = true\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `create-pr`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `boolean`\n\n");
    output.push_str("**Default**: `false`\n\n");
    output.push_str("Whether to create a pull request for the release. Requires both `branch-name` and `push` to be set.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\ncreate-pr = true\n```\n\n");
    output.push_str("---\n\n");

    output.push_str("### `confirm`\n\n");
    output.push_str("**Optional**\n\n");
    output.push_str("**Type**: `boolean`\n\n");
    output.push_str("**Default**: `true`\n\n");
    output.push_str("Whether to prompt for confirmation before making changes.\n\n");
    output.push_str("**Example**:\n\n");
    output.push_str("```toml\n[release]\nconfirm = false\n```\n\n");
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{Args, main};
    use crate::Mode;

    #[test]
    #[cfg(unix)]
    fn configuration_markdown_up_to_date() -> Result<()> {
        main(&Args { mode: Mode::Check })?;
        Ok(())
    }
}
