use crate::error::{PapercutError, Result};
use crate::warnings::{WarningManager, WarningCategory};
use glob::Pattern;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

/// Scan and expand file patterns into actual file paths
pub fn expand_file_patterns(
    path: &Path,
    include_types: &[String],
    exclude_patterns: &[String],
    warning_manager: &Arc<WarningManager>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Check if path contains glob patterns
    let path_str = path.to_str().ok_or_else(|| {
        PapercutError::Config(format!("Invalid path: {}", path.display()))
    })?;

    if contains_glob_pattern(path_str) {
        // Expand glob pattern
        files.extend(expand_glob_pattern(path_str)?);
    } else if path.is_dir() {
        // Scan directory recursively
        files.extend(scan_directory(path, warning_manager)?);
    } else if path.is_file() {
        // Single file
        files.push(path.to_path_buf());
    } else {
        return Err(PapercutError::FileNotFound(path.display().to_string()));
    }

    // Apply filters
    files = apply_file_type_filter(files, include_types);
    files = apply_exclusion_patterns(files, exclude_patterns, warning_manager)?;

    // Sort for consistent output
    files.sort();

    Ok(files)
}

/// Check if a path string contains glob pattern characters
fn contains_glob_pattern(path: &str) -> bool {
    path.contains('*') || path.contains('?') || path.contains('[') || path.contains(']')
}

/// Expand a glob pattern to matching files
fn expand_glob_pattern(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in glob::glob(pattern)
        .map_err(|e| PapercutError::Config(format!("Invalid glob pattern '{}': {}", pattern, e)))?
    {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    files.push(path);
                }
            }
            Err(e) => {
                return Err(PapercutError::Config(format!("Error reading glob entry: {}", e)));
            }
        }
    }

    Ok(files)
}

/// Recursively scan a directory for files
fn scan_directory(dir: &Path, warning_manager: &Arc<WarningManager>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry_result in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
    {
        match entry_result {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                }
            }
            Err(e) => {
                // Warn about directory walking errors (permission denied, broken symlinks, etc.)
                warning_manager.warnf(
                    WarningCategory::Filesystem,
                    format!("Error accessing path during directory scan: {}", e)
                );
            }
        }
    }

    Ok(files)
}

/// Filter files by file type (extension)
fn apply_file_type_filter(files: Vec<PathBuf>, include_types: &[String]) -> Vec<PathBuf> {
    // If no types specified, include all files
    if include_types.is_empty() {
        return files;
    }

    files
        .into_iter()
        .filter(|path| {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                include_types.iter().any(|t| t == ext)
            } else {
                false
            }
        })
        .collect()
}

/// Filter out files matching exclusion patterns
fn apply_exclusion_patterns(
    files: Vec<PathBuf>,
    exclude_patterns: &[String],
    warning_manager: &Arc<WarningManager>,
) -> Result<Vec<PathBuf>> {
    // If no exclusion patterns, return all files
    if exclude_patterns.is_empty() {
        return Ok(files);
    }

    // Compile exclusion patterns
    let patterns: Result<Vec<Pattern>> = exclude_patterns
        .iter()
        .map(|p| {
            Pattern::new(p).map_err(|e| {
                PapercutError::Config(format!("Invalid exclusion pattern '{}': {}", p, e))
            })
        })
        .collect();

    let patterns = patterns?;

    // Filter files that don't match any exclusion pattern
    let filtered = files
        .into_iter()
        .filter(|path| {
            match path.to_str() {
                Some(path_str) => {
                    // Path is valid UTF-8, check against patterns
                    !patterns.iter().any(|pattern| pattern.matches(path_str))
                }
                None => {
                    // Warn about non-UTF-8 path
                    warning_manager.warnf(
                        WarningCategory::Filesystem,
                        format!("Skipping file with non-UTF-8 path: {}", path.display())
                    );
                    // Exclude non-UTF-8 paths as we can't reliably match them
                    false
                }
            }
        })
        .collect();

    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_glob_pattern() {
        assert!(contains_glob_pattern("*.rs"));
        assert!(contains_glob_pattern("src/**/*.py"));
        assert!(contains_glob_pattern("file[12].txt"));
        assert!(contains_glob_pattern("file?.rs"));
        assert!(!contains_glob_pattern("src/main.rs"));
        assert!(!contains_glob_pattern("README.md"));
    }

    #[test]
    fn test_apply_file_type_filter() {
        let files = vec![
            PathBuf::from("main.rs"),
            PathBuf::from("lib.py"),
            PathBuf::from("data.json"),
            PathBuf::from("README.md"),
        ];

        let filtered = apply_file_type_filter(files.clone(), &["rs".to_string(), "py".to_string()]);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("main.rs")));
        assert!(filtered.contains(&PathBuf::from("lib.py")));

        // Empty filter should return all files
        let all = apply_file_type_filter(files, &[]);
        assert_eq!(all.len(), 4);
    }
}
