use cache::Cache;
use failure::Error;
use plantuml_backend::{get_extension, get_image_filename, PlantUMLBackend};
use plantuml_server_backend::PlantUMLServer;
use plantuml_shell_backend::PlantUMLShell;
use plantumlconfig::PlantUMLConfig;
use reqwest::Url;
use std::fs;
use std::cell::UnsafeCell;
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

    //Always create the image output dir
    fs::create_dir_all(&img_root).expect("Failed to create image output dir.");

    if let Ok(server_url) = Url::parse(&cmd) {
        Box::new(PlantUMLServer::new(server_url, img_root.clone()))
    } else {
        Box::new(PlantUMLShell::new(cmd, img_root.clone()))
    }
}

struct CachingBackendDecorator {
    cache: Cache,
    img_root: PathBuf,
    real_backend: Box<dyn PlantUMLBackend>,
}

impl CachingBackendDecorator {
    fn render_from_string(&self, plantuml_code: &String) -> Result<PathBuf, Error> {
        if let Some(cached_image_path) = self.cache.get_entry(plantuml_code) {
            let extension = get_extension(plantuml_code);
            let filename = get_image_filename(&self.img_root, &extension);
            match fs::copy(&cached_image_path, &filename) {
                Ok(_) => {
                    return Ok(filename);
                }
                Err(e) => {
                    bail!(
                        "Failed to copy cached image '{}' to '{}' ({})",
                        cached_image_path.to_string_lossy(),
                        filename.to_string_lossy(),
                        e
                    );
                }
            };
        }

        match self.real_backend.render_from_string(plantuml_code) {
            Ok(img_file_path) => {
                bail!("Not implemented");
                //self.cache.add_entry(plantuml_code, &img_file_path);
                return Ok(img_file_path);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

impl PlantUMLBackend for CachingBackendDecorator {
    fn render_from_string(&self, plantuml_code: &String) -> Result<PathBuf, Error> {
        CachingBackendDecorator::render_from_string(self, plantuml_code)
    }
}
