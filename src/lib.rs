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

use failure::Error;
use markdown_plantuml_pipeline::{render_plantuml_code_blocks, PlantUMLCodeBlockRenderer};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use plantuml_backend::PlantUMLBackend;
use plantumlconfig::PlantUMLConfig;

impl PlantUMLCodeBlockRenderer for Box<PlantUMLBackend> {
    fn render(&self, code_block: String) -> String {
        match self.render_from_string(&code_block) {
            Ok(image_path) => format!("<div><img class='plantuml' src='{}' /></div>\n", image_path),
            Err(e) => {
                error!("Failed to generate PlantUML diagram.");
                String::from(format!("<pre>\nPlantUML rendering error:\n{}</pre>\n", e))
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
                match render_chapter(&plantuml_cmd, &chapter.content) {
                    Ok(md) => chapter.content = md,
                    Err(_) => {
                        return;
                    }
                };
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn render_chapter(
    plantuml_renderer: &impl PlantUMLCodeBlockRenderer,
    chapter: &str,
) -> Result<String, Error> {
    Ok(render_plantuml_code_blocks(chapter, plantuml_renderer))
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
