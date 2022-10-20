use crate::plantuml_backend::PlantUMLBackend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use crate::plantuml_server_backend::PlantUMLServer;
use crate::plantuml_shell_backend::PlantUMLShell;
use crate::plantumlconfig::PlantUMLConfig;
use anyhow::{bail, Result};
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use reqwest::Url;
use std::path::Path;
use std::process::Command;
use std::str;

/// Test if given PlantUML executable is a working one
fn test_plantuml_executable(cmd: &str) -> bool {
    log::debug!("Testing PlantUML command {}", cmd);
    let result = Command::new(cmd).arg("-version").output().map(|output| {
        match str::from_utf8(&output.stdout) {
            Ok(stdout) => {
                if let Some(version) = stdout.lines().next() {
                    log::info!("Detected {}", version);
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                log::warn!("Failed to parse '{}' stdout ({})", cmd, e);
                false
            }
        }
    });

    match result {
        Ok(valid) => valid,
        Err(e) => {
            log::warn!("Test of '{}' failed ({})", cmd, e);
            false
        }
    }
}

fn create_shell_backend(cfg_cmd: &Option<String>) -> Box<dyn PlantUMLBackend> {
    let mut ret: Option<Box<dyn PlantUMLBackend>> = None;

    if let Some(cmd) = cfg_cmd.as_deref() {
        if test_plantuml_executable(cmd) {
            ret = Some(Box::new(PlantUMLShell::new(cmd.to_string())))
        }
    } else {
        let candidates = ["plantuml", "java -jar plantuml.jar"];
        for cmd in candidates {
            if test_plantuml_executable(cmd) {
                ret = Some(Box::new(PlantUMLShell::new(cmd.to_string())));
                break;
            }
        }
    }

    if let Some(backend) = ret {
        backend
    } else {
        let backend = Box::new(PlantUMLNoShellErrorBackend::new(&cfg_cmd));
        log::error!("{}", backend.msg);
        backend
    }
}

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &PlantUMLConfig) -> Box<dyn PlantUMLBackend> {
    if let Some(cmd) = &cfg.plantuml_cmd {
        if let Ok(server_url) = Url::parse(cmd) {
            if cfg!(feature = "plantuml-ssl-server") || cfg!(feature = "plantuml-server") {
                Box::new(PlantUMLServer::new(server_url))
            } else {
                log::error!(
                    "A PlantUML server is configured, but the mdbook-plantuml plugin \
                    is built without server support.\nPlease rebuild/reinstall the \
                    plugin with server support, or configure the plantuml cli tool as \
                    backend. See the the Features section in README.md"
                );
                Box::new(PlantUMLNoServerErrorBackend {})
            }
        } else {
            create_shell_backend(&cfg.plantuml_cmd)
        }
    } else {
        create_shell_backend(&None)
    }
}

struct PlantUMLNoServerErrorBackend;

impl PlantUMLNoServerErrorBackend {
    fn format_message() -> &'static str {
        "A PlantUML server is configured, but the mdbook-plantuml plugin \
                is built without server support.\nPlease rebuild/reinstall the \
                plugin with server support, or configure the plantuml cli tool as \
                backend. See the the Features section in README.md"
    }
}

impl PlantUMLBackend for PlantUMLNoServerErrorBackend {
    /// Display an error message when the user built the plugin without server
    /// support, but does configure a server in book.toml.
    fn render_from_string(&self, _: &str, _: &str, _: &Path) -> Result<Vec<u8>> {
        bail!(PlantUMLNoServerErrorBackend::format_message());
    }
}

struct PlantUMLNoShellErrorBackend {
    msg: String,
}

impl PlantUMLNoShellErrorBackend {
    fn new(cmd: &Option<String>) -> PlantUMLNoShellErrorBackend {
        PlantUMLNoShellErrorBackend {
            msg: format!("PlantUML executable '{}' was not found, either specify one in book.toml, \
                          or make sure the plantuml executable can be found on the path (or by java)"
                          , cmd.as_deref().unwrap_or("")),
        }
    }
}

impl PlantUMLBackend for PlantUMLNoShellErrorBackend {
    /// Display an error message when the user built the plugin without server
    /// support, but does configure a server in book.toml.
    fn render_from_string(&self, _: &str, _: &str, _: &Path) -> Result<Vec<u8>> {
        bail!("{}", self.msg);
    }
}
