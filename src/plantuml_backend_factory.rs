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
                // First line in stdout should be the version number
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

fn create_shell_backend(cfg: &PlantUMLConfig) -> PlantUMLShell {
    let cfg_cmd = cfg.plantuml_cmd.as_deref().unwrap_or("");
    if is_working_plantuml_cmd(&cfg_cmd) {
        return PlantUMLShell::new(cfg_cmd.to_string());
    } else {
        let candidates = ["plantuml", "java -jar plantuml.jar"];
        for cmd in candidates {
            if is_working_plantuml_cmd(cmd) {
                return PlantUMLShell::new(cmd.to_string());
            }
        }
    }

    panic!(
        "PlantUML executable '{}' was not found, either specify one in book.toml, \
            or make sure the plantuml executable can be found on the path (or by java)",
        cfg_cmd
    );
}

fn create_server_backend(cfg: &PlantUMLConfig) -> Option<PlantUMLServer> {
    let server_address = cfg.plantuml_cmd.as_deref().unwrap_or("");
    if !server_address.starts_with("https:") && !server_address.starts_with("http:") {
        return None;
    }

    if !cfg!(feature = "plantuml-ssl-server") && server_address.starts_with("https:") {
        panic!(
            "The PlantUML command '{}' is configured to use a PlantUML SSL server, but the mdbook-plantuml plugin \
            is built without SSL server support.\nPlease rebuild/reinstall the \
            plugin with SSL server support, or configure the plantuml command line tool as \
            backend. See the the Features section in README.md",
            &server_address
        );
    }

    if !cfg!(feature = "plantuml-ssl-server")
        && !cfg!(feature = "plantuml-server")
        && server_address.starts_with("http:")
    {
        panic!(
            "The PlantUML command '{}' is configured to use a PlantUML server, but the mdbook-plantuml plugin \
            is built without server support.\nPlease rebuild/reinstall the \
            plugin with server support, or configure the plantuml command line tool as \
            backend. See the the Features section in README.md",
            &server_address
        );
    }

    #[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
    match Url::parse(&server_address) {
        Ok(server_url) => {
            return Some(PlantUMLServer::new(server_url));
        }
        Err(e) => {
            panic!(
                "The PlantUML command '{}' is an invalid server address ({})",
                server_address, e
            );
        }
    }
}

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &PlantUMLConfig) -> Box<dyn PlantUMLBackend> {
    if let Some(server_backend) = create_server_backend(&cfg) {
        Box::new(server_backend)
    } else {
        Box::new(create_shell_backend(&cfg))
    }
}
