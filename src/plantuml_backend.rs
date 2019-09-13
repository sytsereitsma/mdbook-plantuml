use std::fs;
use std::path::PathBuf;

use failure::Error;
use plantuml_server_backend::PlantUMLServer;
use plantuml_shell_backend::PlantUMLShell;
use plantumlconfig::PlantUMLConfig;
use url::Url;

pub trait PlantUMLBackend {
    /// Render a PlantUML string and return the diagram file path (as a String)
    /// for use in an anchor tag
    fn render_from_string(&self, s: &String) -> Result<String, Error>;
}

/// Create an instance of the PlantUMLBackend
/// For now only a PlantUMLShell instance is created, later server support will be added
pub fn create(cfg: &PlantUMLConfig, book_root: &PathBuf) -> Box<PlantUMLBackend> {
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

    //Always create the image output dir
    let mut img_root = book_root.clone();
    img_root.push("img");
    fs::create_dir_all(&img_root).expect("Failed to create image output dir.");

    if let Ok(server_url) = Url::parse(&cmd) {
        Box::new(PlantUMLServer::new(server_url, img_root))
    } else {
        Box::new(PlantUMLShell::new(cmd, img_root))
    }
}
