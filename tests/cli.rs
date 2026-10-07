use std::fs;
use std::process::Command;

#[test]
fn resolves_config_relative_paths_from_another_working_directory() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    let working_directory = tempfile::tempdir().expect("working directory should be created");
    fs::write(project.path().join("source.rs"), "fn main() {}\n")
        .expect("source should be writable");
    fs::write(
        project.path().join("papercut.yaml"),
        r#"
output:
  mode: single
  directory: output
  filename: result.pdf
files:
  - path: source.rs
"#,
    )
    .expect("config should be writable");

    let status = Command::new(env!("CARGO_BIN_EXE_papercut"))
        .current_dir(working_directory.path())
        .args([
            "--config",
            project
                .path()
                .join("papercut.yaml")
                .to_str()
                .expect("test path should be UTF-8"),
            "--force",
        ])
        .status()
        .expect("papercut should run");

    assert!(status.success());
    assert!(project.path().join("output/result.pdf").is_file());
    assert!(!working_directory.path().join("output/result.pdf").exists());
}

#[test]
fn refuses_noninteractive_overwrite_before_rendering() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    fs::write(project.path().join("source.rs"), "fn main() {}\n")
        .expect("source should be writable");
    fs::create_dir(project.path().join("output")).expect("output directory should be created");
    fs::write(project.path().join("output/result.pdf"), "sentinel")
        .expect("sentinel should be writable");
    fs::write(
        project.path().join("papercut.yaml"),
        r#"
output:
  mode: single
  directory: output
  filename: result.pdf
files:
  - path: source.rs
"#,
    )
    .expect("config should be writable");

    let output = Command::new(env!("CARGO_BIN_EXE_papercut"))
        .args([
            "--config",
            project
                .path()
                .join("papercut.yaml")
                .to_str()
                .expect("test path should be UTF-8"),
        ])
        .output()
        .expect("papercut should run");

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(project.path().join("output/result.pdf"))
            .expect("sentinel should remain readable"),
        "sentinel"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Use --force"));
}

#[test]
fn multiple_mode_generates_distinct_pdfs_for_matching_stems() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join("one")).unwrap();
    fs::create_dir(project.path().join("two")).unwrap();
    fs::write(project.path().join("one/source.rs"), "fn first() {}\n").unwrap();
    fs::write(project.path().join("two/source.rs"), "fn second() {}\n").unwrap();
    fs::write(
        project.path().join("config.yaml"),
        r#"
output:
  mode: multiple
  directory: output
files:
  - path: one/source.rs
  - path: two/source.rs
page:
  size: Legal
  line_numbers: false
  line_number_separator: false
  vertical_borders: false
  wrap_long_lines: false
syntax_highlighting:
  enabled: false
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_papercut"))
        .args([
            "--config",
            project.path().join("config.yaml").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for name in ["source-1.pdf", "source-2.pdf"] {
        assert!(fs::read(project.path().join("output").join(name))
            .unwrap()
            .starts_with(b"%PDF-"));
    }
}
