use crate::plantuml_backend::PlantUMLBackend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use crate::plantuml_server_backend::PlantUMLServer;
use crate::plantuml_shell_backend::PlantUMLShell;
use crate::plantumlconfig::PlantUMLConfig;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use reqwest::Url;

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &PlantUMLConfig) -> Box<dyn PlantUMLBackend> {
    let cmd = match &cfg.plantuml_cmd {
        Some(s) => s.clone(),
        None => {
            if cfg!(target_os = "windows") {
                String::from("java -jar plantuml.jar")
            } else {
                String::from("/usr/bin/plantuml")
            }
        }
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
    Box::new(PlantUMLShell::new(cmd.to_string()))
}
