use std::process::Command;

fn invoke_xtask(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(arguments)
        .env("PATH", "")
        .output()
        .expect("test binary should start")
}

#[test]
fn cli_unknown_command_returns_usage_error() {
    let result = invoke_xtask(&["unknown"]);
    assert_eq!(result.status.code(), Some(2));
}

#[test]
fn cli_extra_argument_returns_usage_error_without_running_command() {
    let result = invoke_xtask(&["export", "unexpected"]);
    assert_eq!(result.status.code(), Some(2));
}

#[test]
fn cli_build_without_build_tool_returns_actionable_error() {
    let result = invoke_xtask(&["build"]);
    assert_eq!(result.status.code(), Some(1));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("cargo-leptos"));
    assert!(error.contains("cargo xtask setup"));
}
