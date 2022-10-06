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

fn autodetect_plantuml() -> String
{
    let candidates = {
        if cfg!(target_os = "windows") {        
            vec![
                "java -jar plantuml.jar",
                "plantuml.exe",
            ]
        } else {
            vec![
                "java -jar plantuml.jar",
                "plantuml",
                "/usr/bin/plantuml",
                "/usr/local/bin/plantuml",
                "~/bin/plantuml",
                "/bin/plantuml",
                "/opt/plantuml",
            ]
        }    
    };

    // Stick to the default, if things go wrong the error is displayed on
    // the rendered pages
    let mut ret = candidates[0].to_string();
    for cmd in &candidates {
        let status = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(cmd)
                .status()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .status()
        };

        if status.is_ok() && status.unwrap().success() {
            ret = cmd.to_string(); 
            break;
        }
    };

    ret
}

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &PlantUMLConfig) -> Box<dyn PlantUMLBackend> {
    let cmd = match cfg.plantuml_cmd.as_deref() {
        Some(v) => v.to_string(),
        None => autodetect_plantuml()
    };

    create_backend(&cmd)
}

#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
fn create_backend(cmd: &str) -> Box<dyn PlantUMLBackend> {
    if let Ok(server_url) = Url::parse(cmd) {
        Box::new(PlantUMLServer::new(server_url))
    } else {
        Box::new(PlantUMLShell::new(cmd.to_string()))
    }
}

#[cfg(not(any(feature = "plantuml-ssl-server", feature = "plantuml-server")))]
fn create_backend(cmd: &str) -> Box<dyn PlantUMLBackend> {
    if cmd.starts_with("http://") || cmd.starts_with("https://") {
        Box::new(PlantUMLNoServerErrorBackend {})
    } else {
        Box::new(PlantUMLShell::new(cmd.to_string()))
    }
}

struct PlantUMLNoServerErrorBackend;
impl PlantUMLBackend for PlantUMLNoServerErrorBackend {
    /// Display an error message when the user built the plugin without server
    /// support, but does configure a server in book.toml.
    fn render_from_string(&self, _: &str, _: &str, _: &Path) -> Result<()> {
        bail!(
            "A PlantUML server is configured, but the mdbook-plantuml plugin \
            is built without server support.\nPlease rebuild/reinstall the \
            plugin with server support, or configure the plantuml cli tool as \
            backend. See the the Features section in README.md"
        );
    }
}
