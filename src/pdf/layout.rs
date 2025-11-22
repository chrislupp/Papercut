use chrono::Local;
use std::path::Path;

/// Variables that can be substituted in headers and footers
pub struct LayoutVariables {
    pub page: usize,
    pub total: usize,
    pub filename: String,
    pub date: String,
}

impl LayoutVariables {
    /// Create new layout variables
    pub fn new(filename: &Path) -> Self {
        Self {
            page: 1,
            total: 1,
            filename: filename
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            date: Local::now().format("%Y-%m-%d").to_string(),
        }
    }

    /// Update page numbers
    pub fn set_page(&mut self, page: usize, total: usize) {
        self.page = page;
        self.total = total;
    }

    /// Substitute variables in a template string
    /// Supports: {page}, {total}, {filename}, {date}
    pub fn substitute(&self, template: &str) -> String {
        template
            .replace("{page}", &self.page.to_string())
            .replace("{total}", &self.total.to_string())
            .replace("{filename}", &self.filename)
            .replace("{date}", &self.date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_variable_substitution() {
        let mut vars = LayoutVariables::new(&PathBuf::from("test.rs"));
        vars.set_page(5, 10);

        assert_eq!(
            vars.substitute("Page {page} of {total}"),
            "Page 5 of 10"
        );
        assert_eq!(
            vars.substitute("File: {filename} - {date}"),
            format!("File: test.rs - {}", vars.date)
        );
    }
}
