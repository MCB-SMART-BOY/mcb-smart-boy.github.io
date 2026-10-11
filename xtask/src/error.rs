use std::{error::Error, fmt, io, process::ExitStatus};

#[derive(Debug)]
pub enum TaskError {
    /// A command could not be started or its output could not be read.
    CommandIo { command: String, source: io::Error },
    /// A command ran but returned an unsuccessful exit status.
    CommandExit {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    /// A command's output was not valid UTF-8.
    CommandUtf8 {
        command: String,
        source: std::string::FromUtf8Error,
    },
    /// Cargo metadata was not valid JSON or lacked a required field.
    Metadata { detail: String },
    /// Cargo metadata returned malformed JSON.
    MetadataJson { source: serde_json::Error },
    /// A required build tool is absent or has an incompatible exact version.
    ToolVersion { tool: &'static str, detail: String },
    /// A required executable is missing from PATH; setup can install it.
    MissingTool {
        tool: &'static str,
        source: io::Error,
    },
    /// The invocation used an unknown or extra argument.
    Usage { detail: String },
    /// An exported page is missing, empty, or inaccessible.
    Page {
        path: std::path::PathBuf,
        source: io::Error,
    },
    /// One or more independent quality gates failed.
    Gates(Vec<(&'static str, TaskError)>),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIo { command, source } => write!(f, "failed to run {command}: {source}"),
            Self::CommandExit {
                command,
                status,
                stderr,
            } => {
                write!(f, "{command} failed with {status}")?;
                if !stderr.is_empty() {
                    write!(f, ": {}", stderr.trim())?;
                }
                Ok(())
            }
            Self::CommandUtf8 { command, source } => {
                write!(f, "failed to decode output of {command}: {source}")
            }
            Self::Metadata { detail } => write!(f, "failed to read Cargo metadata: {detail}"),
            Self::MetadataJson { source } => {
                write!(f, "failed to parse Cargo metadata JSON: {source}")
            }
            Self::ToolVersion { tool, detail } => {
                write!(f, "incompatible {tool}: {detail}; run cargo xtask setup")
            }
            Self::MissingTool { tool, source } => {
                write!(
                    f,
                    "build tool {tool} is not on PATH: {source}; run cargo xtask setup"
                )
            }
            Self::Usage { detail } => write!(
                f,
                "{detail}; usage: cargo xtask [setup|build|dev|export|check|help]"
            ),
            Self::Page { path, source } => {
                write!(
                    f,
                    "failed to verify exported page {}: {source}",
                    path.display()
                )
            }
            Self::Gates(failures) => {
                writeln!(f, "{} independent check gate(s) failed:", failures.len())?;
                for (name, error) in failures {
                    writeln!(f, "  {name}: {error}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for TaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandIo { source, .. } => Some(source),
            Self::MissingTool { source, .. } => Some(source),
            Self::CommandUtf8 { source, .. } => Some(source),
            Self::MetadataJson { source } => Some(source),
            Self::Page { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub type Result<T, E = TaskError> = std::result::Result<T, E>;

pub fn run(command: &mut std::process::Command) -> Result<()> {
    let label = format!("{command:?}");
    let status = command.status().map_err(|source| TaskError::CommandIo {
        command: label.clone(),
        source,
    })?;
    if status.success() {
        return Ok(());
    }
    Err(TaskError::CommandExit {
        command: label,
        status,
        stderr: String::new(),
    })
}

pub fn capture(command: &mut std::process::Command) -> Result<String> {
    let label = format!("{command:?}");
    let output = command.output().map_err(|source| TaskError::CommandIo {
        command: label.clone(),
        source,
    })?;
    if !output.status.success() {
        return Err(TaskError::CommandExit {
            command: label,
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    String::from_utf8(output.stdout).map_err(|source| TaskError::CommandUtf8 {
        command: label,
        source,
    })
}
