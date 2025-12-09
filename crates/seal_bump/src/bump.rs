use anyhow::{Context, Result};
use glob::glob;
use seal_file_change::{FileChange, FileChanges, make_absolute};
use seal_project::{VersionFile, VersionFileTextFormat};
use std::path::Path;

use crate::Version;

pub fn calculate_version_file_changes(
    root: &Path,
    version_files: &[seal_project::VersionFile],
    current_version: &str,
    new_version: &Version,
) -> Result<FileChanges> {
    let mut changes = Vec::new();

    let new_version_str = new_version.to_string();

    for version_file in version_files {
        match version_file {
            VersionFile::Text {
                path,
                format,
                field,
            } => {
                for path in glob(path)?.filter_map(Result::ok) {
                    let absolute_path = make_absolute(root, &path);
                    let old_content = fs_err::read_to_string(&path)?;

                    let new_content = match format {
                        VersionFileTextFormat::Toml => {
                            let toml: toml::Value = toml::from_str(&old_content)?;

                            let field = field.clone().unwrap_or("package.version".to_string());

                            let found_old_version = nested_toml_key(&toml, &field)?;

                            let Some(last_key) = field.split('.').next_back() else {
                                anyhow::bail!("Failed to replace version in {}", path.display())
                            };

                            if found_old_version != current_version {
                                anyhow::bail!(
                                    "Mismatched version in `{}`, expected `{}`, found `{}`",
                                    path.display(),
                                    current_version,
                                    found_old_version
                                )
                            }

                            old_content.replace(
                                &format!("{last_key} = \"{found_old_version}\""),
                                &format!("{last_key} = \"{new_version}\""),
                            )
                        }

                        VersionFileTextFormat::Text => exact_version_replacement(
                            &absolute_path,
                            &old_content,
                            current_version,
                            &new_version_str,
                        )?,
                    };

                    changes.push(FileChange::new(absolute_path, old_content, new_content));
                }

                if changes.is_empty() {
                    anyhow::bail!("No files found for path or glob `{path}`");
                }
            }
            VersionFile::Search { path, search } => {
                for path in glob(path)?.filter_map(Result::ok) {
                    let old_content = fs_err::read_to_string(&path)?;

                    let search_with_current = search.replace("{version}", current_version);
                    let search_with_new = search.replace("{version}", &new_version_str);

                    if !old_content.contains(&search_with_current) {
                        anyhow::bail!(
                            "Search pattern not found in file. Expected: {search_with_current}"
                        );
                    }

                    let new_content = old_content.replace(&search_with_current, &search_with_new);

                    changes.push(FileChange::new(
                        make_absolute(root, &path),
                        old_content,
                        new_content,
                    ));
                }

                if changes.is_empty() {
                    anyhow::bail!("No files found for path or glob `{path}`");
                }
            }
            VersionFile::JustPath { path } | VersionFile::Simple(path) => {
                for path in glob(path)?.filter_map(Result::ok) {
                    let absolute_path = make_absolute(root, &path);
                    let old_content = fs_err::read_to_string(&path)?;

                    let new_content = exact_version_replacement(
                        &absolute_path,
                        &old_content,
                        current_version,
                        &new_version_str,
                    )?;

                    changes.push(FileChange::new(absolute_path, old_content, new_content));
                }

                if changes.is_empty() {
                    anyhow::bail!("No files found for path or glob `{path}`");
                }
            }
        }
    }

    // Seal.toml file change
    let seal_toml_path = root.join("seal.toml");
    let old_seal_toml_content =
        fs_err::read_to_string(&seal_toml_path).context("Failed to read seal.toml")?;

    let old_line = format!(r#"current-version = "{current_version}""#);
    let new_line = format!(r#"current-version = "{new_version}""#);

    if !old_seal_toml_content.contains(&old_line) {
        anyhow::bail!("Could not find current-version = \"{current_version}\" in seal.toml");
    }

    let updated_content = old_seal_toml_content.replace(&old_line, &new_line);

    changes.push(FileChange::new(
        seal_toml_path,
        old_seal_toml_content,
        updated_content,
    ));

    Ok(FileChanges::new(changes))
}

fn exact_version_replacement(
    path: &Path,
    content: &str,
    current_version: &str,
    version_str: &str,
) -> Result<String> {
    if content.contains(current_version) {
        Ok(content.replace(current_version, version_str))
    } else {
        anyhow::bail!(format!(
            "Version `{current_version}` not found in file `{}`",
            path.display()
        ));
    }
}

fn nested_toml_key<'a>(source: &'a toml::Value, key: &str) -> Result<&'a str> {
    let mut current = source;

    for part in key.split('.') {
        match current {
            toml::Value::Table(table) => {
                current = table
                    .get(part)
                    .ok_or_else(|| anyhow::anyhow!("Key `{part}` not found"))?;
            }
            _ => {
                anyhow::bail!("Expected `{part}` to refer to a TOML table")
            }
        }
    }

    match current {
        toml::Value::String(s) => Ok(s.as_str()),
        toml::Value::Integer(i) => Ok(Box::leak(i.to_string().into_boxed_str())),
        toml::Value::Float(f) => Ok(Box::leak(f.to_string().into_boxed_str())),
        toml::Value::Boolean(b) => Ok(Box::leak(b.to_string().into_boxed_str())),
        other => anyhow::bail!("Expected final TOML value to be string-like, got {other:?}"),
    }
}
