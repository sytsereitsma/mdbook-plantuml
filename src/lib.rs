#![warn(unused_extern_crates)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

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
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::fs;
use std::path::Path;

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
        let img_output_dir = &ctx
            .root
            .join(&ctx.config.book.src)
            .join("mdbook-plantuml-img");

        //Always create the image output dir
        if !img_output_dir.exists() {
            if let Err(e) = fs::create_dir_all(&img_output_dir) {
                return Err(mdbook::errors::Error::msg(format!(
                    "Failed to create the image output dir ({}).",
                    e
                )));
            }
        }

        let renderer = PlantUMLRenderer::new(&cfg, img_output_dir);
        let res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                if let Some(chapter_path) = &chapter.path {
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

fn get_relative_img_url(chapter_path: &Path) -> String {
    let nesting_level = chapter_path.components().count();
    let mut rel_image_url = String::new();
    for _ in 1..nesting_level {
        rel_image_url.push_str("../");
    }
    rel_image_url.push_str("mdbook-plantuml-img");

    rel_image_url
}

fn get_plantuml_config(ctx: &PreprocessorContext) -> PlantUMLConfig {
    match ctx.config.get("preprocessor.plantuml") {
        Some(raw) => raw
            .clone()
            .try_into()
            .map_err(|e| {
                warn!(
                    "Failed to get config from book.toml, using default configuration ({}).",
                    e
                );
                e
            })
            .unwrap_or_default(),
        None => PlantUMLConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_relative_img_url() {
        assert_eq!(
            String::from("mdbook-plantuml-img"),
            get_relative_img_url(&PathBuf::from("chapter 1"))
        );

        assert_eq!(
            String::from("../mdbook-plantuml-img"),
            get_relative_img_url(&PathBuf::from("chapter 1/nested 1"))
        );

        assert_eq!(
            String::from("../../mdbook-plantuml-img"),
            get_relative_img_url(&PathBuf::from("chapter 1/nested 1/nested 2"))
        );
    }
}
