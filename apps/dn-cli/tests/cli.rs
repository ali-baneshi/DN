use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write(path: &std::path::Path, name: &str, body: &str) {
    let target = path.join(name);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(target).unwrap();
    file.write_all(body.as_bytes()).unwrap();
}

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dn-cli-int-{prefix}-{stamp}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn cli_unknown_profile_prints_error_and_non_zero() {
    let dir = temp_dir("unknown-profile");
    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args(["scan", dir.to_str().unwrap(), "--profile", "does-not-exist"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error:"));
    assert!(stderr.contains("available profiles"));
}

#[test]
fn cli_scan_and_review_aliases_match_json_shape() {
    let dir = temp_dir("scan-review-alias");
    write(&dir, "main.rs", "fn main() {}\n");

    let scan_out = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--json",
            "--profile",
            "quick",
        ])
        .output()
        .unwrap();
    let review_out = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "review",
            dir.to_str().unwrap(),
            "--json",
            "--profile",
            "quick",
        ])
        .output()
        .unwrap();

    assert!(scan_out.status.success());
    assert!(review_out.status.success());
    let scan_value: serde_json::Value = serde_json::from_slice(&scan_out.stdout).unwrap();
    let review_value: serde_json::Value = serde_json::from_slice(&review_out.stdout).unwrap();
    assert_eq!(scan_value["schema_version"], "2");
    assert_eq!(scan_value["metadata"]["profile"], "quick");
    assert_eq!(review_value["metadata"]["command"], "review");
    assert_eq!(scan_out.status.code(), review_out.status.code());
}

#[test]
fn cli_python_worker_flag_is_reported() {
    let dir = temp_dir("python-worker");
    let profiles = dir.join(".dn/profiles");
    fs::create_dir_all(&profiles).unwrap();

    let profile = r#"
name = "pytest"
[worker]
enabled = true
[rules]
deterministic_rules = []
suspicious_patterns = ["eval("]
[file_selection]
include_binary = true
[limits]
max_file_size_bytes = 4096
max_file_read_bytes = 4096
max_total_bytes = 4096
max_files = 20
"#;
    fs::write(profiles.join("pytest.toml"), profile).unwrap();

    write(&dir, "snippet.py", "eval(1 + 1)\n");

    let with_worker = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "pytest",
            "--python-worker",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(with_worker.status.success());
    let value: serde_json::Value = serde_json::from_slice(&with_worker.stdout).unwrap();
    assert_eq!(value["integrations"]["worker"]["enabled"], true);
    assert!(value["integrations"]["worker"]["mode"]
        .as_str()
        .unwrap_or_default()
        .contains("python"));
}

#[test]
fn cli_custom_local_profile_loading_from_scan_root() {
    let dir = temp_dir("custom-profile-cli");
    let profile_dir = dir.join(".dn/profiles");
    fs::create_dir_all(&profile_dir).unwrap();

    let profile = r#"
name = "my-security"
inherits = "security"
include_hidden = true
"#;
    fs::write(profile_dir.join("my-security.toml"), profile).unwrap();
    write(&dir, ".env", "token=abc123\n");

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "my-security",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["metadata"]["profile"], "my-security");
    let profile_source = value["metadata"]["profile_source"].as_str().unwrap_or("");
    assert!(profile_source.starts_with("file:"));
    assert!(String::from_utf8(output.stdout).unwrap().contains(".env"));
}

#[test]
fn cli_markdown_output_is_renderable() {
    let dir = temp_dir("markdown-cli");
    write(&dir, "main.rs", "fn main() {}\n");

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--markdown",
            "--profile",
            "quick",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("dn-kernel Review Report"));
    assert!(text.contains("Execution Summary"));
    assert!(text.contains("Profile: `quick`"));
}

#[test]
fn cli_json_and_markdown_are_mutually_exclusive() {
    let dir = temp_dir("cli-output-modes");
    write(&dir, "main.rs", "fn main() {}\n");

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "quick",
            "--json",
            "--markdown",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn cli_markdown_reports_empty_findings_cleanly() {
    let dir = temp_dir("markdown-empty");
    write(&dir, "clean.rs", "fn main() {}\n");

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--markdown",
            "--profile",
            "security",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("*No findings were reported for the current profile."));
}

#[test]
fn cli_content_preview_is_present_when_requested() {
    let dir = temp_dir("content-preview-cli");
    write(&dir, "hello.rs", "fn main() { println!(\"hello\"); }\n");

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--content",
            "--profile",
            "quick",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("\"content_preview\""));
}

#[test]
fn cli_hidden_flag_includes_hidden_paths() {
    let dir = temp_dir("cli-hidden");
    write(&dir, "visible.txt", "todo");
    write(&dir, ".env", "password=shh\n");
    write(&dir, ".dn/.keep", "keep");
    fs::create_dir_all(dir.join(".hiddendir")).unwrap();
    write(&dir, ".hiddendir/secret.txt", "password\n");

    let hidden = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "quick",
            "--hidden",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(hidden.status.success());
    let text = String::from_utf8(hidden.stdout).unwrap();
    assert!(text.contains(".env"));
    assert!(text.contains(".hiddendir/secret.txt"));
}

#[test]
fn cli_hidden_default_skips_hidden_paths() {
    let dir = temp_dir("cli-hidden-default");
    write(&dir, "visible.txt", "todo");
    write(&dir, ".env", "password=shh\n");

    let visible_only = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "quick",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(visible_only.status.success());
    let text = String::from_utf8(visible_only.stdout).unwrap();
    assert!(!text.contains(".env"));
}

#[test]
fn cli_malformed_profile_is_clean_error() {
    let dir = temp_dir("malformed-profile-cli");
    let profile_dir = dir.join(".dn/profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    fs::write(profile_dir.join("bad.toml"), "name = bad").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args(["scan", dir.to_str().unwrap(), "--profile", "bad", "--json"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error:"));
}

#[test]
fn cli_fail_on_returns_exit_code_2() {
    let dir = temp_dir("fail-on");
    write(&dir, "secret.txt", "password = \"hello\"\n");

    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "quick",
            "--fail-on",
            "low",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn cli_summary_only_json_empties_files() {
    let dir = temp_dir("summary-only");
    write(&dir, "secret.txt", "password = \"hello\"\n");
    let output = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "scan",
            dir.to_str().unwrap(),
            "--profile",
            "quick",
            "--summary-only",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["metadata"]["summary_only"], true);
    assert_eq!(value["files"].as_array().unwrap().len(), 0);
}

#[test]
fn cli_profiles_list_and_show_work() {
    let dir = temp_dir("profiles-cmds");
    let profile_dir = dir.join(".dn/profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    fs::write(profile_dir.join("custom.toml"), "name = \"custom\"\n").unwrap();

    let list = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args(["profiles", "list", dir.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let list_text = String::from_utf8(list.stdout).unwrap();
    assert!(list_text.contains("custom"));

    let show = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "profiles",
            "show",
            "custom",
            dir.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(show.status.success());
    let value: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(value["profile"]["name"], "custom");
}

#[test]
fn cli_validate_profile_and_doctor_work() {
    let dir = temp_dir("validate-profile");
    let profile_dir = dir.join(".dn/profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    let profile_path = profile_dir.join("custom.toml");
    fs::write(&profile_path, "name = \"custom\"\n").unwrap();

    let validate = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args([
            "validate-profile",
            profile_path.to_str().unwrap(),
            dir.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(validate.status.success());
    let validate_value: serde_json::Value = serde_json::from_slice(&validate.stdout).unwrap();
    assert_eq!(validate_value["valid"], true);

    let doctor = Command::new(env!("CARGO_BIN_EXE_dn-cli"))
        .args(["doctor", dir.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(doctor.status.success());
    let doctor_value: serde_json::Value = serde_json::from_slice(&doctor.stdout).unwrap();
    assert!(doctor_value["diagnostics"].is_array());
}
