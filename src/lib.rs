mod backend;
#[cfg(any(feature = "plantuml-ssl-server", feature = "plantuml-server"))]
mod base64;
mod config;
mod dir_cleaner;
mod markdown_iterator;
mod pipeline;
mod renderer;

use crate::pipeline::render_plantuml_code_blocks;

use crate::config::Config;
use crate::renderer::Renderer;
use anyhow::{Context, Result, bail};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::PreprocessorContext;
use std::fs;

use std::path::{Path, PathBuf};

pub struct Preprocessor;

impl mdbook::preprocess::Preprocessor for Preprocessor {
    fn name(&self) -> &str {
        "plantuml"
    }

    fn run(
        &self,
        ctx: &PreprocessorContext,
        mut book: Book,
    ) -> Result<Book, mdbook::errors::Error> {
        let cfg = plantuml_config(ctx);
        let img_output_dir = image_output_dir(&ctx.root, &ctx.config.book.src, &cfg)?;
        let org_cwd = std::env::current_dir()?;

        let renderer = Renderer::new(&cfg, img_output_dir);
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item
                && let Some(chapter_path) = &chapter.path {
                    log::info!("Processing chapter '{}' ({:?})", chapter.name, chapter_path);
                    let abs_chapter_dir = dunce::canonicalize(&ctx.root).unwrap().join(&ctx.config.book.src).join(chapter_path).parent().unwrap().to_path_buf();

                    // Change the working dir so the PlantUML `!include` directive can be used using relative includes
                    if let Err(e) = std::env::set_current_dir(&abs_chapter_dir) {
                        log::warn!("Failed to change working dir to {:?}, PlantUML might not be able to render includes ({}).", &abs_chapter_dir, e);
                    }
                    log::debug!("Changed working dir to {:?}.", abs_chapter_dir);

                    let rel_image_url = relative_img_url(chapter_path);
                    chapter.content = render_plantuml_code_blocks(&chapter.content, &renderer, &rel_image_url);
                }
        });

        //Restore the current working dir
        std::env::set_current_dir(org_cwd)?;

        // TODO: also return error state for further processing
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn image_output_dir(root: &Path, src_root: &Path, cfg: &Config) -> Result<PathBuf> {
    let img_output_dir: PathBuf = {
        let canonicalized_root =
            dunce::canonicalize(root).with_context(|| "While determining image output dir")?;
        if cfg.use_data_uris {
            // Create the images in the book root dir (unmonitored by the serve command)
            // This way the rendered images can be cached without causing additional
            // rebuilds.
            canonicalized_root.join(".mdbook-plantuml-cache")
        } else {
            // Create the images in the book src dir
            canonicalized_root
                .join(src_root)
                .join("mdbook-plantuml-img")
        }
    };

    log::info!("Image output/cache dir will be {:?}", &img_output_dir);

    // Always create the image output dir
    if !img_output_dir.is_dir() {
        log::debug!("Image output/cache dir does not exists, creating...");
        if let Err(e) = fs::create_dir_all(&img_output_dir) {
            bail!("Failed to create the image output dir ({}).", e);
        }
    }

    Ok(img_output_dir)
}

fn relative_img_url(chapter_path: &Path) -> String {
    let nesting_level = chapter_path.components().count();
    let mut rel_image_url = String::new();
    for _ in 1..nesting_level {
        rel_image_url.push_str("../");
    }
    rel_image_url.push_str("mdbook-plantuml-img");

    rel_image_url
}

pub fn plantuml_config(ctx: &PreprocessorContext) -> Config {
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
    fn test_relative_img_url() {
        assert_eq!(
            String::from("mdbook-plantuml-img"),
            relative_img_url(Path::new("chapter 1"))
        );

        assert_eq!(
            String::from("../mdbook-plantuml-img"),
            relative_img_url(Path::new("chapter 1/nested 1"))
        );

        assert_eq!(
            String::from("../../mdbook-plantuml-img"),
            relative_img_url(Path::new("chapter 1/nested 1/nested 2"))
        );
    }

    #[test]
    fn test_image_output_dir_data_uri() {
        let output_dir = tempdir().unwrap();
        let book_root = output_dir.path().to_path_buf();
        let src_root = output_dir.path().join("src");

        let cfg = Config {
            plantuml_cmd: None,
            clickable_img: false,
            use_data_uris: true, // true = Create book_root/.mdbook-plantuml-cache
            verbose: false,
            piped: false,
        };

        assert_eq!(
            image_output_dir(&book_root, &src_root, &cfg).unwrap(),
            dunce::canonicalize(book_root.as_path().join(".mdbook-plantuml-cache")).unwrap()
        );
        assert!(book_root.as_path().join(".mdbook-plantuml-cache").exists());
        assert!(!src_root.as_path().join("mdbook-plantuml-img").exists());
    }

    #[test]
    fn test_image_output_dir_no_data_uri() {
        let output_dir = tempdir().unwrap();
        let book_root = output_dir.path().to_path_buf();
        let src_root = output_dir.path().join("src");

        let cfg = Config {
            plantuml_cmd: None,
            clickable_img: false,
            use_data_uris: false, // false = Create src_root/.mdbook-plantuml-cache
            verbose: false,
            piped: false,
        };

        assert_eq!(
            image_output_dir(&book_root, &src_root, &cfg).unwrap(),
            src_root.as_path().join("mdbook-plantuml-img")
        );
        assert!(!book_root.as_path().join(".mdbook-plantuml-cache").exists());
        assert!(src_root.as_path().join("mdbook-plantuml-img").exists());
    }

    #[test]
    fn test_image_output_dir_creation_failure() {
        let output_dir = tempdir().unwrap();
        let book_root = output_dir.path().to_path_buf();
        let src_root = output_dir.path().join("src");

        let cfg = Config {
            plantuml_cmd: None,
            clickable_img: false,
            use_data_uris: true, // true = Create book_root/.mdbook-plantuml-cache
            verbose: false,
            piped: false,
        };

        // Create a file with the same name as the directory, this should fail the dir creation
        fs::File::create(book_root.as_path().join(".mdbook-plantuml-cache")).unwrap();
        assert!(image_output_dir(&book_root, &src_root, &cfg).is_err());
    }
}
