mod error;
mod tools;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use error::{Result, TaskError, capture, run};

const WEBSITE_PACKAGE: &str = "mcb-smart-boy";
const WASM_TARGET: &str = "wasm32-unknown-unknown";
const REQUIRED_PAGES: [&str; 3] = ["index.html", "404.html", "writing/index.html"];

fn main() {
    if let Err(error) = execute(env::args_os().skip(1)) {
        eprintln!("{error}");
        let code = match &error {
            TaskError::CommandExit { status, .. } => status.code().unwrap_or(1),
            TaskError::Usage { .. } => 2,
            _ => 1,
        };
        std::process::exit(code);
    }
}

fn execute(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<()> {
    let selected = args.next().unwrap_or_else(|| "help".into());
    if args.next().is_some() {
        return Err(TaskError::Usage {
            detail: "unexpected extra arguments".to_owned(),
        });
    }
    match selected.to_str() {
        Some("help" | "--help" | "-h") => {
            println!("usage: cargo xtask [setup|build|dev|export|check|help]");
            Ok(())
        }
        Some("setup") => tools::setup(),
        Some("build") => build(false, false),
        Some("dev") => build(false, true),
        Some("export") => export(),
        Some("check") => check(),
        _ => Err(TaskError::Usage {
            detail: format!("unknown command {selected:?}"),
        }),
    }
}

fn find_workspace_root() -> &'static Path {
    // The xtask manifest is checked into the workspace's xtask/ directory.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask manifest must be in the workspace root")
}

fn cargo_command(args: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command.current_dir(find_workspace_root()).args(args);
    command
}

fn read_metadata() -> Result<serde_json::Value> {
    let output = capture(&mut cargo_command(&[
        "metadata",
        "--locked",
        "--no-deps",
        "--format-version",
        "1",
    ]))?;
    serde_json::from_str(&output).map_err(|source| TaskError::MetadataJson { source })
}

fn read_target_directory() -> Result<PathBuf> {
    parse_target_directory(&read_metadata()?)
}

fn parse_target_directory(metadata: &serde_json::Value) -> Result<PathBuf> {
    let target = metadata
        .get("target_directory")
        .and_then(serde_json::Value::as_str)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| TaskError::Metadata {
            detail: "target_directory absent or empty".to_owned(),
        })?;
    let path = PathBuf::from(target);
    if !path.is_absolute() {
        return Err(TaskError::Metadata {
            detail: format!("target_directory is not absolute: {target}"),
        });
    }
    Ok(path)
}

fn read_host_triple() -> Result<String> {
    let output = capture(
        Command::new("rustc")
            .current_dir(find_workspace_root())
            .arg("-vV"),
    )?;
    output
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .filter(|host| !host.is_empty() && !host.contains('/') && !host.contains('\\'))
        .map(str::to_owned)
        .ok_or_else(|| TaskError::Metadata {
            detail: "rustc -vV did not report a valid host triple".to_owned(),
        })
}

fn leptos_command(action: &str, is_release: bool) -> Result<(Command, PathBuf)> {
    let workspace = find_workspace_root();
    let target = read_target_directory()?;
    let host = read_host_triple()?;
    let mut command = Command::new("cargo-leptos");
    command.current_dir(workspace).arg(action);
    if is_release {
        command.arg("--release");
    }
    configure_site_environment(&mut command, is_release);
    command
        .env("LEPTOS_BIN_TARGET_DIR", &target)
        .env("LEPTOS_BIN_TARGET_TRIPLE", &host);
    Ok((command, target.join(host)))
}

fn configure_site_environment(command: &mut Command, is_release: bool) {
    let workspace = find_workspace_root();
    let site = if is_release { "site-release" } else { "site" };
    command
        .env("LEPTOS_SITE_ROOT", workspace.join("run").join(site))
        .env("LEPTOS_SITE_PKG_DIR", "pkg")
        .env("LEPTOS_ASSETS_DIR", workspace.join("public"))
        .env("LEPTOS_STYLE_FILE", workspace.join("assets/site.css"))
        .env("LEPTOS_BIN_TARGET", WEBSITE_PACKAGE)
        .env("LEPTOS_BIN_EXE_NAME", WEBSITE_PACKAGE)
        .env("LEPTOS_OUTPUT_NAME", WEBSITE_PACKAGE)
        .env(
            "LEPTOS_HASH_FILES",
            if is_release { "true" } else { "false" },
        )
        .env("LEPTOS_HASH_FILE_NAME", "hash.txt")
        .env("LEPTOS_BIN_CARGO_COMMAND", "cargo");
}

fn build(is_release: bool, is_watch: bool) -> Result<()> {
    tools::check_tools()?;
    let action = if is_watch { "watch" } else { "build" };
    let (mut command, _) = leptos_command(action, is_release)?;
    run(&mut command)
}

fn export() -> Result<()> {
    tools::check_tools()?;
    let (mut command, target) = leptos_command("build", true)?;
    run(&mut command)?;
    // Only use the binary from the build that just succeeded; do not rebuild via cargo run.
    let binary = target
        .join("release")
        .join(format!("{WEBSITE_PACKAGE}{}", env::consts::EXE_SUFFIX));
    let mut export = Command::new(binary);
    for (key, value) in command.get_envs() {
        if let Some(value) = value {
            export.env(key, value);
        }
    }
    export.current_dir(find_workspace_root()).arg("export");
    run(&mut export)
}

fn verify_pages() -> Result<()> {
    for page in REQUIRED_PAGES {
        let path = find_workspace_root().join("run/pages").join(page);
        let metadata = fs::metadata(&path).map_err(|source| TaskError::Page {
            path: path.clone(),
            source,
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(TaskError::Page {
                path,
                source: io::Error::new(io::ErrorKind::InvalidData, "not a nonempty regular file"),
            });
        }
    }
    Ok(())
}

fn check() -> Result<()> {
    let mut failures = Vec::new();
    check_gate(&mut failures, "Rust format", || {
        run(&mut cargo_command(&["fmt", "--all", "--", "--check"]))
    });
    check_gate(&mut failures, "Rust clippy", || {
        run(&mut cargo_command(&[
            "clippy",
            "--locked",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]))
    });
    check_gate(&mut failures, "WASM hydrate check", || {
        run(&mut cargo_command(&[
            "check",
            "--locked",
            "--package",
            WEBSITE_PACKAGE,
            "--no-default-features",
            "--features",
            "hydrate",
            "--lib",
            "--target",
            WASM_TARGET,
        ]))
    });
    check_gate(&mut failures, "SSR and WASM build", || build(false, false));
    check_gate(&mut failures, "Workspace tests", || {
        run(&mut cargo_command(&["test", "--locked", "--workspace"]))
    });
    check_gate(&mut failures, "Static Pages export", || {
        export().and_then(|()| verify_pages())
    });
    if failures.is_empty() {
        Ok(())
    } else {
        Err(TaskError::Gates(failures))
    }
}

fn check_gate(
    failures: &mut Vec<(&'static str, TaskError)>,
    name: &'static str,
    action: impl FnOnce() -> Result<()>,
) {
    println!("\n+++ {name} +++");
    match action() {
        Ok(()) => println!("PASS {name}"),
        Err(error) => {
            eprintln!("FAIL {name}: {error}");
            failures.push((name, error));
        }
    }
}
