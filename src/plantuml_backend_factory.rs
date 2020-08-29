use plantuml_backend::PlantUMLBackend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use plantuml_server_backend::PlantUMLServer;
use plantuml_shell_backend::PlantUMLShell;
use plantumlconfig::PlantUMLConfig;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
use reqwest::Url;
use std::path::PathBuf;

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(cfg: &PlantUMLConfig, img_root: &PathBuf) -> Box<dyn PlantUMLBackend> {
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

    create_backend(&cmd, &img_root)
}

#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
fn create_backend(cmd: &String, img_root: &PathBuf) -> Box<dyn PlantUMLBackend> {
    if let Ok(server_url) = Url::parse(cmd) {
        Box::new(PlantUMLServer::new(server_url, img_root.clone()))
    } else {
        Box::new(PlantUMLShell::new(cmd.clone(), img_root.clone()))
    }
}

#[cfg(not(any(feature = "plantuml-ssl-server", feature = "plantuml-server")))]
fn create_backend(cmd: &String, img_root: &PathBuf) -> Box<dyn PlantUMLBackend> {
    Box::new(PlantUMLShell::new(cmd.clone(), img_root.clone()))
}
