use std::{
    error::Error,
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

fn cli_binary() -> &'static str {
    env!("CARGO_BIN_EXE_cli")
}

fn run_cli(arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new(cli_binary()).args(arguments).output()?)
}

fn workspace_file(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(path),
    )?
    .to_string_lossy()
    .into_owned())
}

fn temporary_root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("apigee-forge-cli-{name}-{timestamp}"));
    fs::create_dir(&path)?;
    Ok(path)
}

#[test]
fn rejects_missing_template_without_source_leak() -> Result<(), Box<dyn Error>> {
    let root = temporary_root("missing-template")?;
    let spec = workspace_file("DOC_PROJECT/openapi.exemple.yaml")?;
    let missing_template = root.join("missing.json");
    let output_directory = root.join("output");
    let archive = root.join("proxy.zip");
    let output = run_cli(&[
        "--json",
        "generate",
        "--spec",
        &spec,
        "--template",
        missing_template.to_str().ok_or("invalid path")?,
        "--proxy-name",
        "orders",
        "--output",
        output_directory.to_str().ok_or("invalid path")?,
        "--archive",
        archive.to_str().ok_or("invalid path")?,
    ])?;
    assert_eq!(output.status.code(), Some(6));
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["error"]["code"], "FILESYSTEM_ERROR");
    assert!(!String::from_utf8(output.stdout)?.contains("missing.json"));
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_invalid_openapi_without_writing() -> Result<(), Box<dyn Error>> {
    let root = temporary_root("invalid-openapi")?;
    let spec = root.join("invalid.yaml");
    fs::write(&spec, "openapi: [invalid")?;
    let template = workspace_file("schemas/template.example.json")?;
    let output = run_cli(&[
        "--json",
        "generate",
        "--spec",
        spec.to_str().ok_or("invalid path")?,
        "--template",
        &template,
        "--proxy-name",
        "orders",
        "--output",
        root.join("output").to_str().ok_or("invalid path")?,
        "--archive",
        root.join("proxy.zip").to_str().ok_or("invalid path")?,
    ])?;
    assert_eq!(output.status.code(), Some(1));
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["error"]["code"], "INVALID_INPUT");
    assert!(!root.join("output").exists());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_existing_output_without_partial_bundle() -> Result<(), Box<dyn Error>> {
    let root = temporary_root("existing-output")?;
    fs::create_dir(root.join("output"))?;
    fs::create_dir(root.join("output").join("apiproxy"))?;
    let spec = workspace_file("DOC_PROJECT/openapi.exemple.yaml")?;
    let template = workspace_file("schemas/template.example.json")?;
    let output = run_cli(&[
        "--json",
        "generate",
        "--spec",
        &spec,
        "--template",
        &template,
        "--proxy-name",
        "orders",
        "--output",
        root.join("output").to_str().ok_or("invalid path")?,
        "--archive",
        root.join("proxy.zip").to_str().ok_or("invalid path")?,
    ])?;
    assert_eq!(output.status.code(), Some(6));
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["error"]["code"], "FILESYSTEM_ERROR");
    assert!(!root.join("proxy.zip").exists());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_missing_auth_mode_without_prompt_and_returns_json_on_stdout(
) -> Result<(), Box<dyn Error>> {
    let output = run_cli(&["--json", "login"])?;
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["ok"], false);
    assert_eq!(document["error"]["code"], "INVALID_AUTH_MODE");
    Ok(())
}

#[test]
fn rejects_invalid_arguments_with_usage_exit_code_and_json() -> Result<(), Box<dyn Error>> {
    let output = run_cli(&["--json", "generate"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["ok"], false);
    assert_eq!(document["command"], "parse");
    assert_eq!(document["error"]["code"], "INVALID_ARGUMENTS");
    Ok(())
}

#[test]
fn generates_bundle_non_interactively_with_stable_json_output() -> Result<(), Box<dyn Error>> {
    let root = temporary_root("generate")?;
    let output_directory = root.join("directory");
    let archive = root.join("proxy.zip");
    let spec = workspace_file("DOC_PROJECT/openapi.exemple.yaml")?;
    let template = workspace_file("schemas/template.example.json")?;
    let output = run_cli(&[
        "--json",
        "generate",
        "--spec",
        &spec,
        "--template",
        &template,
        "--proxy-name",
        "orders-subprocess",
        "--output",
        output_directory.to_str().ok_or("invalid output path")?,
        "--archive",
        archive.to_str().ok_or("invalid archive path")?,
    ])?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["ok"], true);
    assert_eq!(document["command"], "generate");
    assert_eq!(document["data"]["rendered_file_count"], 7);
    assert!(archive.is_file());
    assert!(output_directory.join("apiproxy").is_dir());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_missing_mode_for_network_commands_without_prompt() -> Result<(), Box<dyn Error>> {
    for arguments in [
        vec!["--json", "list-proxies"],
        vec![
            "--json",
            "deploy",
            "--environment",
            "prod",
            "--proxy-name",
            "orders",
            "--bundle",
            "bundle.zip",
        ],
        vec![
            "--json",
            "status",
            "--environment",
            "prod",
            "--proxy-name",
            "orders",
            "--revision",
            "1",
        ],
    ] {
        let output = run_cli(&arguments)?;
        assert_eq!(output.status.code(), Some(3));
        assert!(output.stderr.is_empty());
        let document: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(document["ok"], false);
        assert_eq!(document["error"]["code"], "INVALID_AUTH_MODE");
    }
    Ok(())
}

#[test]
fn keeps_human_errors_on_stderr_without_json_noise() -> Result<(), Box<dyn Error>> {
    let output = run_cli(&["login"])?;
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("select exactly one explicit authentication mode"));
    Ok(())
}
