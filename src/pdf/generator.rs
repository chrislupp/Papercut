// Papercut - Source code to PDF converter
// Copyright (C) 2025-2026 Christopher A. Lupp
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// Distribution A: This work has been cleared for public release,
// distribution unlimited, case number: AFRL-2026-0405. The views expressed
// are those of the authors and do not reflect the official guidance or
// position of the United States Government, the Department of Defense or of
// the United States Air Force.
//
// Statement from DoD: The Appearance of external hyperlinks does not
// constitute endorsement by the United States Department of Defense (DoD) of
// the linked websites, of the information, products, or services contained
// therein. The DoD does not exercise any editorial, security, or other
// control over the information you may find at these locations.

use crate::config::{Config, OutputMode};
use crate::error::{PapercutError, Result};
use crate::warnings::WarningManager;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn should_overwrite_file(path: &Path, force: bool) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    if force {
        return Ok(true);
    }

    // Check if stdout is a TTY (interactive terminal)
    if !is_terminal::IsTerminal::is_terminal(&io::stdout()) {
        return Err(PapercutError::InvalidConfig(format!(
            "File '{}' already exists. Use --force to overwrite files in non-interactive mode.",
            path.display()
        )));
    }

    // Interactive mode: prompt user
    print!(
        "File '{}' already exists. Overwrite? [y/N]: ",
        path.display()
    );
    io::stdout().flush().map_err(PapercutError::Io)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(PapercutError::Io)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn multiple_output_paths(config: &Config) -> Vec<PathBuf> {
    let mut totals = HashMap::new();
    for file in &config.expanded_files {
        let stem = file
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("output")
            .to_string();
        *totals.entry(stem).or_insert(0usize) += 1;
    }

    let mut occurrences = HashMap::new();
    config
        .expanded_files
        .iter()
        .map(|file| {
            let stem = file
                .path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("output")
                .to_string();
            let occurrence = occurrences.entry(stem.clone()).or_insert(0usize);
            *occurrence += 1;
            let filename = if totals[&stem] > 1 {
                format!("{}-{}.pdf", stem, occurrence)
            } else {
                format!("{}.pdf", stem)
            };
            config.output.directory.join(filename)
        })
        .collect()
}

pub fn generate(
    config: Config,
    verbose: bool,
    force: bool,
    warnings: Arc<WarningManager>,
) -> Result<()> {
    let outputs = match config.output.mode {
        OutputMode::Single => vec![config.output.directory.join(&config.output.filename)],
        OutputMode::Multiple => multiple_output_paths(&config),
    };
    // Check all destinations before doing expensive typesetting.
    let selected = outputs
        .iter()
        .map(|p| should_overwrite_file(p, force))
        .collect::<Result<Vec<_>>>()?;
    let progress = indicatif::ProgressBar::new(outputs.len() as u64);
    for (index, (path, selected)) in outputs.iter().zip(selected).enumerate() {
        if selected {
            let mut document = config.clone();
            if config.output.mode == OutputMode::Multiple {
                document.expanded_files = vec![config.expanded_files[index].clone()];
                // Preserve multiple mode's source-only output.
                document.cover_page.enabled = false;
                document.markdown_report.enabled = false;
            }
            if verbose {
                println!("Typesetting {} with Typst", path.display());
            }
            let pdf = super::typst_renderer::render(&document, &warnings)?;
            fs::write(path, pdf)?;
            println!("✓ Generated: {}", path.display());
        } else {
            println!("Skipping file: {}", path.display());
        }
        progress.inc(1);
    }
    progress.finish_and_clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disambiguates_duplicate_output_stems() {
        let mut config: Config = serde_saphyr::from_str(
            r#"
output:
  mode: multiple
  directory: output
files:
  - path: one/config.rs
"#,
        )
        .expect("config structure should deserialize");
        config.expanded_files = vec![
            crate::config::ExpandedFileEntry {
                path: PathBuf::from("one/config.rs"),
                title: None,
            },
            crate::config::ExpandedFileEntry {
                path: PathBuf::from("two/config.rs"),
                title: None,
            },
        ];

        assert_eq!(
            multiple_output_paths(&config),
            vec![
                PathBuf::from("output/config-1.pdf"),
                PathBuf::from("output/config-2.pdf")
            ]
        );
    }
}
