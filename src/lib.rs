#![warn(unused_extern_crates)]
#[macro_use]
extern crate log;
extern crate deflate;
extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate reqwest;
extern crate uuid;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate simulacrum;
#[cfg(test)]
extern crate tempfile;

mod base64_plantuml;
mod markdown_plantuml_pipeline;
mod plantuml_backend;
mod plantuml_server_backend;
mod plantuml_shell_backend;
mod plantumlconfig;

use markdown_plantuml_pipeline::{render_plantuml_code_blocks, PlantUMLCodeBlockRenderer};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use plantuml_backend::PlantUMLBackend;
use plantumlconfig::PlantUMLConfig;
use std::path::PathBuf;

impl PlantUMLCodeBlockRenderer for Box<PlantUMLBackend> {
    fn render(&self, code_block: String, rel_img_url: &String) -> String {
        match self.render_from_string(&code_block, rel_img_url) {
            Ok(image_path) => format!("![{}]({})\n\n", image_path, image_path),
            Err(e) => {
                error!("Failed to generate PlantUML diagram.");
                String::from(format!("\nPlantUML rendering error:\n{}\n\n", e))
            }
        }
    }
}

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
        let plantuml_cmd = plantuml_backend::create(&cfg, &ctx.config.build.build_dir);

        let res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                let rel_image_url = get_relative_img_url(&chapter.path);
                chapter.content =
                    render_plantuml_code_blocks(&chapter.content, &plantuml_cmd, &rel_image_url);
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn get_relative_img_url(chapter_path: &PathBuf) -> String {
    let nesting_level = chapter_path.components().count();
    let mut rel_image_url = String::new();
    for _ in 1..nesting_level {
        rel_image_url.push_str("../");
    }
    rel_image_url.push_str("img");

    rel_image_url
}

fn get_plantuml_config(ctx: &PreprocessorContext) -> PlantUMLConfig {
    match ctx.config.get("preprocessor.plantuml") {
        Some(raw) => raw
            .clone()
            .try_into()
            .or_else(|e| {
                warn!(
                    "Failed to get config from book.toml, using default configuration ({}).",
                    e
                );
                Err(e)
            })
            .unwrap_or(PlantUMLConfig::default()),
        None => PlantUMLConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_relative_img_url() {
        assert_eq!(
            String::from("img"),
            get_relative_img_url(&PathBuf::from("chapter 1"))
        );

        assert_eq!(
            String::from("../img"),
            get_relative_img_url(&PathBuf::from("chapter 1/nested 1"))
        );

        assert_eq!(
            String::from("../../img"),
            get_relative_img_url(&PathBuf::from("chapter 1/nested 1/nested 2"))
        );
    }
}
