use crate::plantuml_backend::PlantUMLBackend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use crate::plantuml_server_backend::PlantUMLServer;
use crate::plantuml_shell_backend::PlantUMLShell;
use crate::plantumlconfig::PlantUMLConfig;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use reqwest::Url;
use std::process::Command;
use std::str;

/// Test if given PlantUML executable is a working one
fn is_working_plantuml_cmd(cmd: &str) -> bool {
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
        if is_working_plantuml_cmd(cmd) {
            ret = Some(Box::new(PlantUMLShell::new(cmd.to_string())))
        }
    } else {
        let candidates = ["plantuml", "java -jar plantuml.jar"];
        for cmd in candidates {
            if is_working_plantuml_cmd(cmd) {
                ret = Some(Box::new(PlantUMLShell::new(cmd.to_string())));
                break;
            }
        }
    }

    if let Some(backend) = ret {
        backend
    } else {
        panic!("PlantUML executable '{}' was not found, either specify one in book.toml, \
                or make sure the plantuml executable can be found on the path (or by java)"
                , cfg_cmd.as_deref().unwrap_or(""));
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
                panic!(
                    "A PlantUML server is configured, but the mdbook-plantuml plugin \
                    is built without server support.\nPlease rebuild/reinstall the \
                    plugin with server support, or configure the plantuml cli tool as \
                    backend. See the the Features section in README.md"
                );
            }
        } else {
            create_shell_backend(&cfg.plantuml_cmd)
        }
    } else {
        create_shell_backend(&None)
    }
}
