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
