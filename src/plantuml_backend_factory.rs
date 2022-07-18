use crate::plantuml_backend::PlantUMLBackend;
#[cfg(feature = "exp-cmdline-pipe")]
use crate::plantuml_cmdline_backend::PlantUMLExecutableBackend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use crate::plantuml_server_backend::PlantUMLServer;
#[cfg(not(feature = "exp-cmdline-pipe"))]
use crate::plantuml_shell_backend::PlantUMLShell;
use crate::plantumlconfig::PlantUMLConfig;
use anyhow::{bail, Result};
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use reqwest::Url;
use std::path::Path;

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &PlantUMLConfig) -> Box<dyn PlantUMLBackend> {
    let cmd = cfg.plantuml_cmd.as_deref().unwrap_or({
        if cfg!(target_os = "windows") {
            "java -jar plantuml.jar"
        } else {
            "/usr/bin/plantuml"
        }
    });

    create_backend(cmd)
}

#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
fn create_backend(cmd: &str) -> Box<dyn PlantUMLBackend> {
    if let Ok(server_url) = Url::parse(cmd) {
        Box::new(PlantUMLServer::new(server_url))
    } else {
        create_executable_backend(cmd)
    }
}

#[cfg(not(any(feature = "plantuml-ssl-server", feature = "plantuml-server")))]
fn create_backend(cmd: &str) -> Box<dyn PlantUMLBackend> {
    if cmd.starts_with("http://") || cmd.starts_with("https://") {
        Box::new(PlantUMLNoServerErrorBackend {})
    } else {
        create_executable_backend(cmd)
    }
}

#[cfg(not(feature = "exp-cmdline-pipe"))]
fn create_executable_backend(cmd: &str) -> Box<dyn PlantUMLBackend> {
    Box::new(PlantUMLShell::new(cmd.to_string()))
}

#[cfg(feature = "exp-cmdline-pipe")]
fn create_executable_backend(cmd: &str) -> Box<dyn PlantUMLBackend> {
    Box::new(PlantUMLExecutableBackend::new(cmd.to_string(), None))
}

struct PlantUMLNoServerErrorBackend;
impl PlantUMLBackend for PlantUMLNoServerErrorBackend {
    /// Display an error message when the user built the plugin without server
    /// support, but does configure a server in book.toml.
    fn render_from_string(&self, _: &str, _: &str, _: &str, _: &Path) -> Result<()> {
        bail!(
            "A PlantUML server is configured, but the mdbook-plantuml plugin \
            is built without server support.\nPlease rebuild/reinstall the \
            plugin with server support, or configure the plantuml cli tool as \
            backend. See the the Features section in README.md"
        );
    }
}
