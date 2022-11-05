#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
mod base64_plantuml;
mod dir_cleaner;
mod markdown_plantuml_pipeline;
mod plantuml_backend;
mod plantuml_backend_factory;
mod plantuml_renderer;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
mod plantuml_server_backend;
mod plantuml_shell_backend;
mod plantumlconfig;
mod util;

use crate::markdown_plantuml_pipeline::render_plantuml_code_blocks;

use crate::plantuml_renderer::PlantUMLRenderer;
use crate::plantumlconfig::PlantUMLConfig;
use anyhow::{bail, Result};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::fs;
use std::path::{Path, PathBuf};

pub struct PlantUMLPreprocessor;

impl Preprocessor for PlantUMLPreprocessor {
    fn name(&self) -> &str {
        "plantuml"
    }

    fn run(
        &self,
        ctx: &PreprocessorContext,
        mut book: Book,
    ) -> Result<Book, mdbook::errors::Error> {
        let cfg = get_plantuml_config(ctx);
        let img_output_dir = get_image_output_dir(&ctx.root, &ctx.config.book.src, &cfg)?;

        let renderer = PlantUMLRenderer::new(&cfg, img_output_dir);
        let res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                if let Some(chapter_path) = &chapter.path {
                    log::info!("Processing chapter '{}' ({:?})", chapter.name, chapter_path);

                    let rel_image_url = get_relative_img_url(chapter_path);
                    chapter.content =
                        render_plantuml_code_blocks(&chapter.content, &renderer, &rel_image_url);
                }
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn get_image_output_dir(
    root: &PathBuf,
    src_root: &PathBuf,
    cfg: &PlantUMLConfig,
) -> Result<PathBuf> {
    let img_output_dir = {
        if cfg.use_data_uris {
            // Create the images in the book root dir (unmonitored by the serve command)
            // This way the rendered images can be cached without causing additional
            // rebuilds.
            root.join(".mdbook-plantuml-cache")
        } else {
            // Create the images in the book src dir
            root.join(&src_root).join("mdbook-plantuml-img")
        }
    };

    // Always create the image output dir
    if !img_output_dir.is_dir() {
        if let Err(e) = fs::create_dir_all(&img_output_dir) {
            bail!("Failed to create the image output dir ({}).", e);
        }
    }

    Ok(img_output_dir)
}

fn get_relative_img_url(chapter_path: &Path) -> String {
    let nesting_level = chapter_path.components().count();
    let mut rel_image_url = String::new();
    for _ in 1..nesting_level {
        rel_image_url.push_str("../");
    }
    rel_image_url.push_str("mdbook-plantuml-img");

    rel_image_url
}

pub fn get_plantuml_config(ctx: &PreprocessorContext) -> PlantUMLConfig {
    ctx.config
        .get("preprocessor.plantuml")
        .and_then(|raw| {
            raw.clone()
                .try_into()
                .map_err(|e| {
                    log::warn!(
                        "Failed to get config from book.toml, using default configuration ({}).",
                        e
                    );
                    e
                })
                .ok()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_get_relative_img_url() {
        assert_eq!(
            String::from("mdbook-plantuml-img"),
            get_relative_img_url(Path::new("chapter 1"))
        );

        assert_eq!(
            String::from("../mdbook-plantuml-img"),
            get_relative_img_url(Path::new("chapter 1/nested 1"))
        );

        assert_eq!(
            String::from("../../mdbook-plantuml-img"),
            get_relative_img_url(Path::new("chapter 1/nested 1/nested 2"))
        );
    }

    #[test]
    fn test_get_image_output_dir_data_uri() {
        let output_dir = tempdir().unwrap();
        let book_root = output_dir.path().to_path_buf();
        let src_root = output_dir.path().join("src");

        let cfg = PlantUMLConfig {
            plantuml_cmd: None,
            clickable_img: false,
            use_data_uris: true, // true = Create book_root/.mdbook-plantuml-cache
            verbose: false,
        };

        assert_eq!(
            get_image_output_dir(&book_root, &src_root, &cfg).unwrap(),
            book_root.as_path().join(".mdbook-plantuml-cache")
        );
        assert!(book_root.as_path().join(".mdbook-plantuml-cache").exists());
        assert!(!src_root.as_path().join("mdbook-plantuml-img").exists());
    }

    #[test]
    fn test_get_image_output_dir_no_data_uri() {
        let output_dir = tempdir().unwrap();
        let book_root = output_dir.path().to_path_buf();
        let src_root = output_dir.path().join("src");

        let cfg = PlantUMLConfig {
            plantuml_cmd: None,
            clickable_img: false,
            use_data_uris: false, // false = Create src_root/.mdbook-plantuml-cache
            verbose: false,
        };

        assert_eq!(
            get_image_output_dir(&book_root, &src_root, &cfg).unwrap(),
            src_root.as_path().join("mdbook-plantuml-img")
        );
        assert!(!book_root.as_path().join(".mdbook-plantuml-cache").exists());
        assert!(src_root.as_path().join("mdbook-plantuml-img").exists());
    }

    #[test]
    fn test_get_image_output_dir_creation_failure() {
        let output_dir = tempdir().unwrap();
        let book_root = output_dir.path().to_path_buf();
        let src_root = output_dir.path().join("src");

        let cfg = PlantUMLConfig {
            plantuml_cmd: None,
            clickable_img: false,
            use_data_uris: true, // true = Create book_root/.mdbook-plantuml-cache
            verbose: false,
        };

        // Create a file with the same name as the directory, this should fail the dir creation
        fs::File::create(&book_root.as_path().join(".mdbook-plantuml-cache")).unwrap();
        assert!(get_image_output_dir(&book_root, &src_root, &cfg).is_err());
    }
}
