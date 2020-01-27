use cache::Cache;
use failure::Error;
use plantuml_backend::{get_extension, get_image_filename, PlantUMLBackend};
use plantuml_server_backend::PlantUMLServer;
use plantuml_shell_backend::PlantUMLShell;
use plantumlconfig::PlantUMLConfig;
use reqwest::Url;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

/// Create an instance of the PlantUMLBackend
/// # Arguments
/// * `img_root` - The path to the directory where to store the images
/// * `cfg` - The configuration options
pub fn create(
    cfg: &PlantUMLConfig,
    img_root: &PathBuf,
    book_dir: &PathBuf,
) -> Box<dyn PlantUMLBackend> {
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

    let mut backend: Box<dyn PlantUMLBackend>;
    if let Ok(server_url) = Url::parse(&cmd) {
        backend = Box::new(PlantUMLServer::new(server_url, img_root.clone()));
    } else {
        backend = Box::new(PlantUMLShell::new(cmd, img_root.clone()));
    }

    if cfg.enable_cache.unwrap_or(false) {
        let cache_dir = {
            if let Some(c) = &cfg.cache_dir {
                PathBuf::from(c)
            } else {
                book_dir.join(".plantuml-cache")
            }
        };

        match Cache::new(&cache_dir, cfg.clean_cache.unwrap_or(true)) {
            Ok(cache) => {
                backend = Box::new(CachingBackendDecorator {
                    cache: RefCell::new(cache),
                    img_root: img_root.clone(),
                    real_backend: backend,
                });
            }
            Err(e) => {
                eprintln!("Failed to instantiate cache ({}), cache is disabled!", e);
            }
        };
    }

    backend
}

/// A backend that tries to load a cached image first and calls the real PlantUML
/// backend when that fails.
struct CachingBackendDecorator {
    /// The image cache
    cache: RefCell<Cache>,
    /// The path where to save the images to
    img_root: PathBuf,
    /// The fallback backend to use when a cache entry is not found
    real_backend: Box<dyn PlantUMLBackend>,
}

impl CachingBackendDecorator {
    fn render_from_string(&self, plantuml_code: &String) -> Result<PathBuf, Error> {
        if let Some(cached_image_path) = self.cache.borrow().get_entry(plantuml_code) {
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
                self.cache
                    .borrow_mut()
                    .add_entry(plantuml_code, &img_file_path);
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
