use crate::backend::Backend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use crate::backend::server::PlantUMLServer;
use crate::backend::shell::{PlantUMLShell, split_shell_command};
use crate::config::Config;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use reqwest::Url;
use std::process::Command;
use std::str;

/// Test if given PlantUML executable is a working one
fn is_working_plantuml_cmd(cmd: &str) -> bool {
    let cmd_parts = match split_shell_command(cmd) {
        Ok(cp) => cp,
        Err(e) => {
            log::warn!("PlantUML command {} is invalid ({}).", cmd, e);
            return false;
        }
    };

    log::error!("Testing PlantUML command {} ({:?})", cmd, cmd_parts);
    let result = Command::new(&cmd_parts[0])
        .args(&cmd_parts[1..])
        .arg("-version")
        .output()
        .map(|output| {
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
                    log::error!("Failed to parse '{}' stdout ({})", cmd, e);
                    false
                }
            }
        });

    match result {
        Ok(valid) => valid,
        Err(e) => {
            log::error!("Test of '{}' failed ({})", cmd, e);
            false
        }
    }
}

fn create_shell_backend(cfg: &Config) -> PlantUMLShell {
    let piped = cfg.piped;
    if let Some(cfg_cmd) = &cfg.plantuml_cmd {
        if is_working_plantuml_cmd(cfg_cmd) {
            PlantUMLShell::new(cfg_cmd.to_string(), piped)
        } else {
            panic!(
                "PlantUML executable '{}' was not found, please check the plantuml-cmd in book.toml, \
                    or make sure the plantuml executable can be found on the path (or by java)",
                cfg_cmd
            );
        }
    } else {
        let candidates = ["plantuml", "java -jar plantuml.jar"];
        for cmd in candidates {
            if is_working_plantuml_cmd(cmd) {
                return PlantUMLShell::new(cmd.to_string(), piped);
            }
        }

        panic!(
            "PlantUML executable could not be auto detected, tried '{}'. either specify one in book.toml, \
                or make sure the plantuml executable can be found on the path (or by java)",
            candidates.join(",")
        );
    }
}

/// Checks if a plantuml server is configured, but the application is built without server support
/// Panics if the configured PlantUML server address is incompatible with the build features.
fn check_server_support(server_address: &str) {
    if !server_address.starts_with("https:") && !server_address.starts_with("http:") {
        return;
    }

    assert!(
        cfg!(feature = "plantuml-ssl-server") || !server_address.starts_with("https:"),
        "The PlantUML command '{}' is configured to use a PlantUML SSL server, but the mdbook-plantuml plugin \
        is built without SSL server support.\nPlease rebuild/reinstall the \
        plugin with SSL server support, or configure the plantuml command line tool as \
        backend. See the the Features section in README.md",
        &server_address
    );

    assert!(
        cfg!(feature = "plantuml-ssl-server")
            || cfg!(feature = "plantuml-server")
            || !server_address.starts_with("http:"),
        "The PlantUML command '{}' is configured to use a PlantUML server, but the mdbook-plantuml plugin \
        is built without server support.\nPlease rebuild/reinstall the \
        plugin with server support, or configure the plantuml command line tool as \
        backend. See the the Features section in README.md",
        &server_address
    );
}

#[cfg(not(any(feature = "plantuml-ssl-server", feature = "plantuml-server")))]
/// Returns None, or panics, because we have no server support
/// Returns Option<PlantUMLShell>, because otherwise a dummy trait would need to be implemented as a placeholder
fn create_server_backend(cfg: &Config) -> Option<PlantUMLShell> {
    let server_address = cfg.plantuml_cmd.as_deref().unwrap_or("");
    check_server_support(server_address);

    None
}

#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
fn create_server_backend(cfg: &Config) -> Option<PlantUMLServer> {
    let server_address = cfg.plantuml_cmd.as_deref().unwrap_or("");
    if !server_address.starts_with("https:") && !server_address.starts_with("http:") {
        return None;
    }

    // Make sure the application was built with the appropriate features (in this case potential https support)
    check_server_support(server_address);

    match Url::parse(server_address) {
        Ok(server_url) => Some(PlantUMLServer::new(server_url)),
        Err(e) => {
            panic!(
                "The PlantUML command '{}' is an invalid server address ({})",
                server_address, e
            );
        }
    }
}

/// Create an instance of the Backend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &Config) -> Box<dyn Backend> {
    if let Some(server_backend) = create_server_backend(cfg) {
        Box::new(server_backend)
    } else {
        Box::new(create_shell_backend(cfg))
    }
}
