use std::process::Command;

use crate::error::{Result, TaskError, capture, run};

struct Tool {
    name: &'static str,
    package: &'static str,
    version: &'static str,
    reported_version: &'static str,
    install_flags: &'static [&'static str],
}

const TOOLS: [Tool; 3] = [
    Tool {
        name: "cargo-leptos",
        package: "cargo-leptos",
        version: "0.3.11",
        reported_version: "cargo-leptos 0.3.11",
        install_flags: &["--features", "no_downloads"],
    },
    Tool {
        name: "wasm-bindgen",
        package: "wasm-bindgen-cli",
        version: "0.2.129",
        reported_version: "wasm-bindgen 0.2.129",
        install_flags: &[],
    },
    Tool {
        name: "wasm-opt",
        package: "wasm-opt",
        version: "0.116.1",
        reported_version: "wasm-opt version 116 (version_116)",
        install_flags: &["--no-default-features"],
    },
];

pub fn setup() -> Result<()> {
    validate_bindgen_dependency()?;
    run(Command::new("rustup")
        .current_dir(crate::find_workspace_root())
        .args([
            "component",
            "add",
            "rustfmt",
            "clippy",
            "--toolchain",
            "nightly",
        ]))?;
    run(Command::new("rustup")
        .current_dir(crate::find_workspace_root())
        .args([
            "target",
            "add",
            "wasm32-unknown-unknown",
            "--toolchain",
            "nightly",
        ]))?;
    for tool in &TOOLS {
        if !is_installed(tool)? {
            let mut install = crate::cargo_command(&[
                "install",
                tool.package,
                "--version",
                tool.version,
                "--locked",
                "--force",
            ]);
            install.args(tool.install_flags);
            run(&mut install)?;
        }
        check_tool(tool)?;
    }
    Ok(())
}

pub fn check_tools() -> Result<()> {
    for tool in &TOOLS {
        check_tool(tool)?;
    }
    validate_bindgen_dependency()
}

fn is_installed(tool: &Tool) -> Result<bool> {
    match check_tool(tool) {
        Ok(()) => Ok(true),
        Err(TaskError::ToolVersion { .. } | TaskError::MissingTool { .. }) => Ok(false),
        Err(error) => Err(error),
    }
}

fn check_tool(tool: &Tool) -> Result<()> {
    let mut command = Command::new(tool.name);
    command
        .current_dir(crate::find_workspace_root())
        .arg("--version");
    let actual = match capture(&mut command) {
        Ok(output) => output,
        Err(TaskError::CommandIo { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Err(TaskError::MissingTool {
                tool: tool.name,
                source,
            });
        }
        Err(error) => return Err(error),
    };
    if !matches_version(&actual, tool) {
        return Err(TaskError::ToolVersion {
            tool: tool.name,
            detail: format!(
                "expected {}, found {:?}",
                tool.reported_version,
                actual.trim()
            ),
        });
    }
    if tool.name == "wasm-opt" && !has_installed_package(tool)? {
        return Err(TaskError::ToolVersion {
            tool: tool.name,
            detail: format!(
                "expected cargo install package {} v{}",
                tool.package, tool.version
            ),
        });
    }
    Ok(())
}

fn matches_version(output: &str, tool: &Tool) -> bool {
    output.trim() == tool.reported_version
}

fn has_installed_package(tool: &Tool) -> Result<bool> {
    let listing = capture(&mut crate::cargo_command(&["install", "--list"]))?;
    Ok(lists_package(&listing, tool))
}

fn lists_package(listing: &str, tool: &Tool) -> bool {
    let expected = format!("{} v{}:", tool.package, tool.version);
    listing.lines().any(|line| line == expected)
}

fn validate_bindgen_dependency() -> Result<()> {
    let metadata = crate::read_metadata()?;
    let requirement = metadata["packages"]
        .as_array()
        .and_then(|packages| {
            packages
                .iter()
                .find(|package| package["name"] == crate::WEBSITE_PACKAGE)
        })
        .and_then(|package| package["dependencies"].as_array())
        .and_then(|dependencies| {
            dependencies
                .iter()
                .find(|dependency| dependency["name"] == "wasm-bindgen")
        })
        .and_then(|dependency| dependency["req"].as_str())
        .ok_or_else(|| TaskError::Metadata {
            detail: "website wasm-bindgen dependency is missing".to_owned(),
        })?;
    let tool = &TOOLS[1];
    if requirement != format!("={}", tool.version) {
        return Err(TaskError::ToolVersion {
            tool: tool.name,
            detail: format!(
                "website requires {requirement}, tool specification requires ={}; update the matching tool specification",
                tool.version
            ),
        });
    }
    Ok(())
}
