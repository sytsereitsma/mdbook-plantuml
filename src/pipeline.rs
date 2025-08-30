use crate::markdown_iterator::{Block, MarkdownIterator};
use crate::renderer::RendererTrait;
use std::string::String;

enum ProcessedBlockResult<'a> {
    Slice(&'a str),
    Full(String),
}

pub fn render_plantuml_code_blocks(
    markdown: &str,
    renderer: &impl RendererTrait,
    rel_image_url: &str,
) -> String {
    let mut processed = String::new();

    let markdown_iterator = MarkdownIterator::new(markdown);
    markdown_iterator.for_each(
        |block| match &process_block(&block, renderer, rel_image_url) {
            ProcessedBlockResult::Slice(s) => processed.push_str(s),
            ProcessedBlockResult::Full(s) => processed.push_str(s),
        },
    );

    processed
}

fn process_block<'a>(
    block: &'a Block,
    renderer: &impl RendererTrait,
    rel_image_url: &str,
) -> ProcessedBlockResult<'a> {
    match block {
        Block::Text(text_block) => ProcessedBlockResult::Slice(text_block.text),
        Block::Code(code_block) => {
            if code_block.info_string.is_plantuml() {
                let image_format = code_block.get_image_format();
                let rendered =
                    renderer.render(code_block.code, rel_image_url, image_format.to_string());
                match rendered {
                    Ok(data) => ProcessedBlockResult::Full(data),
                    Err(e) => {
                        log::error!("{}", e);
                        ProcessedBlockResult::Full(format!("{e}"))
                    }
                }
            } else {
                ProcessedBlockResult::Full(code_block.full_block.to_string())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::cell::RefCell;

    struct FakeRenderer {
        pub code: RefCell<Option<String>>,
        pub rel_image_url: RefCell<Option<String>>,
        pub image_format: RefCell<Option<String>>,
    }

    impl FakeRenderer {
        pub fn new() -> Self {
            Self {
                code: RefCell::new(None),
                rel_image_url: RefCell::new(None),
                image_format: RefCell::new(None),
            }
        }
    }

    impl RendererTrait for FakeRenderer {
        fn render(&self, code: &str, rel_image_url: &str, image_format: String) -> Result<String> {
            self.code.replace(Some(code.to_string()));
            self.rel_image_url.replace(Some(rel_image_url.to_string()));
            self.image_format.replace(Some(image_format));

            Ok(String::from("\nFake renderer was here\n"))
        }
    }

    #[test]
    fn test_render_plantuml_code_blocks() {
        let markdown = r#"Some text before
```plantuml
plantuml code
```
Some text after
```rust
fn main() {
    println!("Hello, world!");
}
"#;

        let renderer = FakeRenderer::new();

        let rel_image_url = "images/plantuml";

        let result = render_plantuml_code_blocks(markdown, &renderer, rel_image_url);
        assert_eq!(
            renderer.code.borrow().as_ref(),
            Some(&String::from("plantuml code\n"))
        );
        assert_eq!(
            renderer.rel_image_url.borrow().as_ref(),
            Some(&String::from("images/plantuml"))
        );
        assert_eq!(
            renderer.image_format.borrow().as_ref(),
            Some(&String::from("svg"))
        );

        assert_eq!(
            result,
            "Some text before\n\nFake renderer was here\nSome text after\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n"
        );
    }
}
