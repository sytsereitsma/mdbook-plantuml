use cache::Cache;
use dir_cleaner::DirCleaner;
use plantuml_backend::{get_image_filename, PlantUMLBackend};
use plantuml_backend_factory;
use plantumlconfig::PlantUMLConfig;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

pub trait PlantUMLRendererTrait {
    fn render(&self, plantuml_code: &String, rel_img_url: &String) -> String;
}

pub struct PlantUMLRenderer {
    backend: Box<dyn PlantUMLBackend>,
    cache: Option<RefCell<Cache>>,
    cleaner: RefCell<DirCleaner>,
    img_root: PathBuf,
}

impl PlantUMLRenderer {
    pub fn new(cfg: &PlantUMLConfig, img_root: &PathBuf, book_dir: &PathBuf) -> PlantUMLRenderer {
        let mut renderer = PlantUMLRenderer {
            backend: plantuml_backend_factory::create(cfg, img_root),
            cache: None,
            cleaner: RefCell::new(DirCleaner::new(img_root)),
            img_root: img_root.clone(),
        };

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
                    renderer.cache = Some(RefCell::new(cache));
                }
                Err(e) => {
                    error!(
                        "Failed to instantiate cache ({}), cache will be disabled!",
                        e
                    );
                }
            };
        }

        renderer
    }

    fn get_from_cache(&self, plantuml_code: &String) -> Option<PathBuf> {
        if self.cache.is_none() {
            return None;
        }

        if let Some(cached_image_path) = self
            .cache
            .as_ref()
            .unwrap()
            .borrow()
            .get_entry(plantuml_code)
        {
            let filename = get_image_filename(&self.img_root, plantuml_code);

            // Avoid an mdBook rebuild loop, only copy if the file is not there
            // already
            if !filename.exists() {
                match fs::copy(&cached_image_path, &filename) {
                    Ok(_) => {
                        return Some(filename);
                    }
                    Err(e) => {
                        error!(
                            "Failed to copy cached image '{}' to '{}' ({})",
                            cached_image_path.to_string_lossy(),
                            filename.to_string_lossy(),
                            e
                        );
                    }
                };
            } else {
                return Some(filename);
            }
        }

        None
    }

    fn create_md_link(rel_img_url: &String, image_path: &PathBuf) -> String {
        let img_url = format!(
            "{}/{}",
            rel_img_url,
            image_path.file_name().unwrap().to_str().unwrap()
        );
        format!("![]({})\n\n", img_url)
    }

    pub fn render(&self, plantuml_code: &String, rel_img_url: &String) -> String {
        if let Some(img_file_path) = self.get_from_cache(plantuml_code) {
            self.cleaner.borrow_mut().keep(&img_file_path);
            PlantUMLRenderer::create_md_link(rel_img_url, &img_file_path)
        } else {
            // Image not cached, so generate and add cache entry
            match self.backend.render_from_string(plantuml_code) {
                Ok(img_file_path) => {
                    self.cleaner.borrow_mut().keep(&img_file_path);
                    if self.cache.is_some() {
                        self.cache
                            .as_ref()
                            .unwrap()
                            .borrow_mut()
                            .add_entry(plantuml_code, &img_file_path);
                    }
                    PlantUMLRenderer::create_md_link(rel_img_url, &img_file_path)
                }
                Err(e) => {
                    error!("Failed to generate PlantUML diagram.");
                    String::from(format!("\nPlantUML rendering error:\n{}\n\n", e))
                }
            }
        }
    }
}

impl PlantUMLRendererTrait for PlantUMLRenderer {
    fn render(&self, plantuml_code: &String, rel_img_url: &String) -> String {
        PlantUMLRenderer::render(self, plantuml_code, rel_img_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_create_md_link() {
        assert_eq!(
            String::from("![](foo/bar/baz.svg)\n\n"),
            PlantUMLRenderer::create_md_link(
                &String::from("foo/bar"),
                &PathBuf::from("/froboz/baz.svg")
            )
        );

        assert_eq!(
            String::from("![](/baz.svg)\n\n"),
            PlantUMLRenderer::create_md_link(&String::from(""), &PathBuf::from("baz.svg"))
        );

        assert_eq!(
            String::from("![](/baz.svg)\n\n"),
            PlantUMLRenderer::create_md_link(&String::from(""), &PathBuf::from("foo/baz.svg"))
        );
    }

    struct BackendMock {
        is_ok: bool,
    }

    impl PlantUMLBackend for BackendMock {
        fn render_from_string(
            &self,
            plantuml_code_doubling_as_return_type: &String,
        ) -> Result<PathBuf, Error> {
            if self.is_ok {
                return Ok(PathBuf::from(plantuml_code_doubling_as_return_type));
            }
            bail!("Oh no")
        }
    }

    #[test]
    fn test_rendering() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cache: None,
            cleaner: RefCell::new(DirCleaner::new(&output_dir.path().to_path_buf())),
            img_root: PathBuf::from(output_dir.path().to_path_buf()),
        };

        assert_eq!(
            String::from("![](rel/url/image.svg)\n\n"),
            renderer.render(&String::from("some/image.svg"), &String::from("rel/url"))
        );
    }

    #[test]
    fn test_rendering_failure() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: false }),
            cache: None,
            cleaner: RefCell::new(DirCleaner::new(&output_dir.path().to_path_buf())),
            img_root: PathBuf::from(output_dir.path().to_path_buf()),
        };

        assert_eq!(
            String::from("\nPlantUML rendering error:\nOh no\n\n"),
            renderer.render(&String::from(""), &String::from("rel/url"))
        );
    }
}
