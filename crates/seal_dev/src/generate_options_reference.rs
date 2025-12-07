//! Generate a Markdown-compatible listing of configuration options for `pyproject.toml`.
//!
//! Based on: <https://github.com/astral-sh/ruff/blob/dc8db1afb08704ad6a788c497068b01edf8b460d/crates/ruff_dev/src/generate_options.rs>
use std::fmt::Write;
use std::path::PathBuf;

use anstream::println;
use anyhow::{Result, bail};
use itertools::Itertools;
use pretty_assertions::StrComparison;

use seal_options_metadata::{Field, OptionSet, OptionsMetadata, Visit};
use seal_project::Config;

use crate::{Mode, ROOT_DIR};

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, default_value_t, value_enum)]
    pub(crate) mode: Mode,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let reference_string = generate();
    let filename = "configuration.md";
    let reference_path = PathBuf::from(ROOT_DIR)
        .join("docs")
        .join("reference")
        .join(filename);

    match args.mode {
        Mode::DryRun => {
            println!("{reference_string}");
        }
        Mode::Check => match fs_err::read_to_string(reference_path) {
            Ok(current) => {
                if current == reference_string {
                    println!("Up-to-date: {filename}");
                } else {
                    let comparison = StrComparison::new(&current, &reference_string);
                    bail!(
                        "{filename} changed, please run `cargo dev generate-options-reference`:\n{comparison}"
                    );
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                bail!("{filename} not found, please run `cargo dev generate-options-reference`");
            }
            Err(err) => {
                bail!(
                    "{filename} changed, please run `cargo dev generate-options-reference`:\n{err}"
                );
            }
        },
        Mode::Write => match fs_err::read_to_string(&reference_path) {
            Ok(current) => {
                if current == reference_string {
                    println!("Up-to-date: {filename}");
                } else {
                    println!("Updating: {filename}");
                    fs_err::write(reference_path, reference_string.as_bytes())?;
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                println!("Updating: {filename}");
                fs_err::write(reference_path, reference_string.as_bytes())?;
            }
            Err(err) => {
                bail!(
                    "{filename} changed, please run `cargo dev generate-options-reference`:\n{err}"
                );
            }
        },
    }

    Ok(())
}

enum OptionType {
    Configuration,
}

fn generate() -> String {
    let mut output = String::new();

    generate_set(
        &mut output,
        Set::Global {
            set: Config::metadata(),
            option_type: OptionType::Configuration,
        },
        &mut Vec::new(),
    );

    output
}

fn generate_set(output: &mut String, set: Set, parents: &mut Vec<Set>) {
    match &set {
        Set::Global { option_type, .. } => {
            let header = match option_type {
                OptionType::Configuration => "## Configuration\n",
            };
            output.push_str(header);
        }
        Set::Named { name, .. } => {
            let title = parents
                .iter()
                .filter_map(|set| set.name())
                .chain(std::iter::once(name.as_str()))
                .join(".");
            writeln!(output, "### `{title}`\n").unwrap();

            if let Some(documentation) = set.metadata().documentation() {
                output.push_str(documentation);
                output.push('\n');
                output.push('\n');
            }
        }
    }

    let mut visitor = CollectOptionsVisitor::default();
    set.metadata().record(&mut visitor);

    let (mut fields, mut sets) = (visitor.fields, visitor.groups);

    fields.sort_unstable_by(|(name, _), (name2, _)| name.cmp(name2));
    sets.sort_unstable_by(|(name, _), (name2, _)| name.cmp(name2));

    parents.push(set);

    // Generate the fields.
    for (name, field) in &fields {
        emit_field(output, name, field, parents.as_slice());
        output.push_str("---\n\n");
    }

    // Generate all the sub-sets.
    for (set_name, sub_set) in &sets {
        generate_set(
            output,
            Set::Named {
                name: set_name.to_owned(),
                set: *sub_set,
            },
            parents,
        );
    }

    parents.pop();
}

enum Set {
    Global {
        option_type: OptionType,
        set: OptionSet,
    },
    Named {
        name: String,
        set: OptionSet,
    },
}

impl Set {
    fn name(&self) -> Option<&str> {
        match self {
            Self::Global { .. } => None,
            Self::Named { name, .. } => Some(name),
        }
    }

    fn metadata(&self) -> &OptionSet {
        match self {
            Self::Global { set, .. } => set,
            Self::Named { set, .. } => set,
        }
    }
}

#[allow(clippy::format_push_string)]
fn emit_field(output: &mut String, name: &str, field: &Field, parents: &[Set]) {
    let header_level = if parents.len() > 1 { "####" } else { "###" };
    let parents_anchor = parents.iter().filter_map(|parent| parent.name()).join("_");

    if parents_anchor.is_empty() {
        output.push_str(&format!("{header_level} [`{name}`](#{name})\n"));
    } else {
        output.push_str(&format!(
            "{header_level} [`{name}`](#{parents_anchor}_{name})\n"
        ));

        // the anchor used to just be the name, but now it's the group name
        // for backwards compatibility, we need to keep the old anchor
        output.push_str(&format!("<span id=\"{name}\"></span>\n"));
    }

    output.push('\n');

    if let Some(deprecated) = &field.deprecated {
        output.push_str("!!! warning \"Deprecated\"\n");
        output.push_str("    This option has been deprecated");

        if let Some(since) = deprecated.since {
            write!(output, " in {since}").unwrap();
        }

        output.push('.');

        if let Some(message) = deprecated.message {
            writeln!(output, " {message}").unwrap();
        }

        output.push('\n');
    }

    output.push_str(field.doc);
    output.push_str("\n\n");
    if let Some(default) = field.default {
        output.push_str(format!("**Default value**: `{}`\n", default).as_str());
    } else {
        output.push_str("**Required**\n");
    }
    output.push('\n');
    if let Some(possible_values) = field
        .possible_values
        .as_ref()
        .filter(|values| !values.is_empty())
    {
        output.push_str("**Possible values**:\n\n");
        for value in possible_values {
            output.push_str(format!("- {value}\n").as_str());
        }
    } else {
        output.push_str(&format!("**Type**: `{}`\n", field.value_type));
    }
    output.push('\n');
    output.push_str("**Example usage**:\n\n");

    if let Set::Global {
        option_type: OptionType::Configuration,
        ..
    } = parents[0]
    {
        output.push_str(&format_tab(
            "seal.toml",
            &format_header(field.scope, field.example, parents),
            field.example,
        ));
    }
    output.push('\n');
}

fn format_tab(tab_name: &str, header: &str, content: &str) -> String {
    if header.is_empty() {
        format!(
            "=== \"{}\"\n\n    ```toml\n{}\n    ```\n",
            tab_name,
            textwrap::indent(content, "    ")
        )
    } else {
        format!(
            "=== \"{}\"\n\n    ```toml\n    {}\n{}\n    ```\n",
            tab_name,
            header,
            textwrap::indent(content, "    ")
        )
    }
}

/// Format the TOML header for the example usage for a given option.
///
/// For example: `[tool.uv.pip]`.
fn format_header(scope: Option<&str>, example: &str, parents: &[Set]) -> String {
    let header = None
        .into_iter()
        .chain(parents.iter().filter_map(|parent| parent.name()))
        .chain(scope)
        .join(".");

    // Ex) `[[tool.uv.index]]`
    if example.starts_with(&format!("[[{header}")) {
        return String::new();
    }
    // Ex) `[tool.uv.sources]`
    if example.starts_with(&format!("[{header}")) {
        return String::new();
    }

    if header.is_empty() {
        String::new()
    } else {
        format!("[{header}]")
    }
}

#[derive(Default)]
struct CollectOptionsVisitor {
    groups: Vec<(String, OptionSet)>,
    fields: Vec<(String, Field)>,
}

impl Visit for CollectOptionsVisitor {
    fn record_set(&mut self, name: &str, group: OptionSet) {
        self.groups.push((name.to_owned(), group));
    }

    fn record_field(&mut self, name: &str, field: Field) {
        self.fields.push((name.to_owned(), field));
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Result;

    use crate::Mode;

    use super::{Args, main};

    #[test]
    fn test_generate_options_reference() -> Result<()> {
        main(&Args { mode: Mode::Check })
    }
}
