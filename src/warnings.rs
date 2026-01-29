// Papercut - Source code to PDF converter
// Copyright (C) 2026 Papercut Contributors
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

use std::collections::HashSet;
use std::io::Write;
use std::sync::Mutex;

/// Categories of warnings that can be selectively enabled/disabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningCategory {
    Fonts,
    Themes,
    Highlighting,
    Filesystem,
}

impl WarningCategory {
    /// Parse a category from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fonts" => Some(Self::Fonts),
            "themes" => Some(Self::Themes),
            "highlighting" => Some(Self::Highlighting),
            "filesystem" => Some(Self::Filesystem),
            _ => None,
        }
    }

    /// Get the string representation of a category
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fonts => "fonts",
            Self::Themes => "themes",
            Self::Highlighting => "highlighting",
            Self::Filesystem => "filesystem",
        }
    }
}

/// Manages warning output with configurable categories
pub struct WarningManager {
    enabled: bool,
    silenced_categories: Mutex<HashSet<WarningCategory>>,
}

impl WarningManager {
    /// Create a new warning manager with warnings enabled
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            silenced_categories: Mutex::new(HashSet::new()),
        }
    }

    /// Silence multiple categories at once
    pub fn silence_categories(&self, categories: &[WarningCategory]) {
        if let Ok(mut silenced) = self.silenced_categories.lock() {
            for category in categories {
                silenced.insert(*category);
            }
        }
    }

    /// Check if a category is silenced
    fn is_category_silenced(&self, category: WarningCategory) -> bool {
        if let Ok(silenced) = self.silenced_categories.lock() {
            silenced.contains(&category)
        } else {
            false
        }
    }

    /// Emit a warning to stderr if enabled and not silenced
    pub fn warn(&self, category: WarningCategory, message: &str) {
        if !self.enabled || self.is_category_silenced(category) {
            return;
        }

        let category_str = category.as_str();
        let _ = writeln!(
            std::io::stderr(),
            "\x1b[33mWarning\x1b[0m [\x1b[36m{}\x1b[0m]: {}",
            category_str, message
        );
    }

    /// Emit a warning with formatted arguments
    pub fn warnf(&self, category: WarningCategory, message: String) {
        self.warn(category, &message);
    }
}

impl Default for WarningManager {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_str() {
        assert_eq!(WarningCategory::from_str("fonts"), Some(WarningCategory::Fonts));
        assert_eq!(WarningCategory::from_str("Fonts"), Some(WarningCategory::Fonts));
        assert_eq!(WarningCategory::from_str("themes"), Some(WarningCategory::Themes));
        assert_eq!(WarningCategory::from_str("highlighting"), Some(WarningCategory::Highlighting));
        assert_eq!(WarningCategory::from_str("filesystem"), Some(WarningCategory::Filesystem));
        assert_eq!(WarningCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_warning_manager_enabled() {
        let manager = WarningManager::new(true);
        // Can't test actual stderr output in unit tests easily,
        // but we can verify the manager is created
        assert!(manager.enabled);
    }

    #[test]
    fn test_warning_manager_disabled() {
        let manager = WarningManager::new(false);
        assert!(!manager.enabled);
    }

    #[test]
    fn test_silence_single_category() {
        let manager = WarningManager::new(true);
        manager.silence_categories(&[WarningCategory::Fonts]);
        assert!(manager.is_category_silenced(WarningCategory::Fonts));
        assert!(!manager.is_category_silenced(WarningCategory::Themes));
    }

    #[test]
    fn test_silence_multiple_categories() {
        let manager = WarningManager::new(true);
        manager.silence_categories(&[WarningCategory::Fonts, WarningCategory::Themes]);
        assert!(manager.is_category_silenced(WarningCategory::Fonts));
        assert!(manager.is_category_silenced(WarningCategory::Themes));
        assert!(!manager.is_category_silenced(WarningCategory::Highlighting));
    }
}
