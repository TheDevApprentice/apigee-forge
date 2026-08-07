use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{json, Value};

#[derive(Debug, Clone)]
struct MatrixCase {
    name: &'static str,
    arguments: Vec<&'static str>,
    expected_exit_code: i32,
    expected_error_code: &'static str,
}

fn run_case(case: &MatrixCase) -> Result<Value, Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(&case.arguments)
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(case.expected_exit_code),
        "{}",
        case.name
    );
    assert!(output.stderr.is_empty(), "{} wrote to stderr", case.name);
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["ok"], false);
    assert_eq!(document["error"]["code"], case.expected_error_code);
    Ok(json!({
        "name": case.name,
        "exit_code": output.status.code(),
        "json_error_code": document["error"]["code"],
        "stdout_json": true,
        "stderr_empty": true
    }))
}

#[derive(Default)]
struct FakeOutputRunner {
    results: Vec<Value>,
}

impl FakeOutputRunner {
    fn record(&mut self, result: Value) {
        self.results.push(result);
    }
}

fn temporary_directory() -> Result<PathBuf, Box<dyn Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("apigee-forge-command-matrix-{timestamp}"));
    fs::create_dir(&path)?;
    Ok(path)
}

#[test]
fn covers_non_interactive_command_error_matrix() -> Result<(), Box<dyn Error>> {
    let root = temporary_directory()?;
    let dangerous_template = root.join("../outside-template");
    let cases = vec![
        MatrixCase {
            name: "login_requires_explicit_mode",
            arguments: vec!["--json", "login"],
            expected_exit_code: 3,
            expected_error_code: "INVALID_AUTH_MODE",
        },
        MatrixCase {
            name: "list_proxies_requires_explicit_mode",
            arguments: vec!["--json", "list-proxies"],
            expected_exit_code: 3,
            expected_error_code: "INVALID_AUTH_MODE",
        },
        MatrixCase {
            name: "deploy_requires_explicit_mode",
            arguments: vec![
                "--json",
                "deploy",
                "--environment",
                "prod",
                "--proxy-name",
                "orders",
                "--bundle",
                "bundle.zip",
            ],
            expected_exit_code: 3,
            expected_error_code: "INVALID_AUTH_MODE",
        },
        MatrixCase {
            name: "status_requires_explicit_mode",
            arguments: vec![
                "--json",
                "status",
                "--environment",
                "prod",
                "--proxy-name",
                "orders",
                "--revision",
                "1",
            ],
            expected_exit_code: 3,
            expected_error_code: "INVALID_AUTH_MODE",
        },
        MatrixCase {
            name: "unknown_command_rejected",
            arguments: vec!["--json", "unknown-command"],
            expected_exit_code: 2,
            expected_error_code: "INVALID_ARGUMENTS",
        },
    ];
    let mut runner = FakeOutputRunner::default();
    for case in &cases {
        runner.record(run_case(case)?);
    }
    let mut results = runner.results;

    let output = Command::new(env!("CARGO_BIN_EXE_cli"))
        .args([
            "--json",
            "template",
            "--directory",
            root.to_str().ok_or("invalid temporary path")?,
            "show",
            dangerous_template
                .to_str()
                .ok_or("invalid dangerous path")?,
        ])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let document: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(document["error"]["code"], "INVALID_INPUT");
    results.push(json!({
        "name": "dangerous_template_name_rejected",
        "exit_code": output.status.code(),
        "json_error_code": document["error"]["code"],
        "stdout_json": true,
        "stderr_empty": true
    }));

    let report_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("test-results");
    fs::create_dir_all(&report_directory)?;
    let report_path = report_directory.join("cli_command_matrix.json");
    serde_json::to_writer_pretty(
        File::create(&report_path)?,
        &json!({
            "test": "covers_non_interactive_command_error_matrix",
            "cases": results,
            "secrets_in_output": false
        }),
    )?;
    eprintln!("test report: {}", report_path.display());
    fs::remove_dir_all(root)?;
    Ok(())
}
