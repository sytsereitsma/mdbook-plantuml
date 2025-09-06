use crate::markdown_iterator::{Block, MarkdownIterator};
use crate::renderer::RendererTrait;
use anyhow::Result;
use std::string::String;
enum ProcessedBlockResult<'a> {
    Slice(&'a str),
    Full(String),
}

pub fn render_plantuml_code_blocks(
    markdown: &str,
    renderer: &impl RendererTrait,
    rel_image_url: &str,
    fail_on_error: bool, // When false, errors in rendering are logged and written to the book, otherwise they cause the book generation to fail
) -> Result<String> {
    let mut processed = String::new();

    let markdown_iterator = MarkdownIterator::new(markdown);
    for block in markdown_iterator {
        match process_block(&block, renderer, rel_image_url, fail_on_error)? {
            ProcessedBlockResult::Slice(s) => processed.push_str(s),
            ProcessedBlockResult::Full(s) => processed.push_str(&s),
        }
    }

    Ok(processed)
}

fn process_block<'a>(
    block: &'a Block,
    renderer: &impl RendererTrait,
    rel_image_url: &str,
    fail_on_error: bool,
) -> Result<ProcessedBlockResult<'a>> {
    match block {
        Block::Text(text_block) => Ok(ProcessedBlockResult::Slice(text_block.text)),
        Block::Code(code_block) => {
            if code_block.info_string.is_plantuml() {
                let image_format = code_block.get_image_format();
                let rendered =
                    renderer.render(code_block.code, rel_image_url, image_format.to_string());

                match rendered {
                    Ok(data) => Ok(ProcessedBlockResult::Full(data)),
                    Err(e) => {
                        if fail_on_error {
                            Err(e)
                        } else {
                            log::error!("Rendering error: {}", e);
                            Ok(ProcessedBlockResult::Full(format!("{e}")))
                        }
                    }
                }
            } else {
                Ok(ProcessedBlockResult::Full(
                    code_block.full_block.to_string(),
                ))
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
        pub result: Option<String>, // Can't clone Result, so use Option (None is error)
    }

    impl FakeRenderer {
        pub fn new() -> Self {
            Self {
                code: RefCell::new(None),
                rel_image_url: RefCell::new(None),
                image_format: RefCell::new(None),
                result: Some(String::from("\nFake renderer was here\n")),
            }
        }
    }

    impl RendererTrait for FakeRenderer {
        fn render(&self, code: &str, rel_image_url: &str, image_format: String) -> Result<String> {
            self.code.replace(Some(code.to_string()));
            self.rel_image_url.replace(Some(rel_image_url.to_string()));
            self.image_format.replace(Some(image_format));

            match self.result {
                Some(ref s) => Ok(s.clone()),
                None => Err(anyhow::anyhow!("FakeRenderer error")),
            }
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

        let result = render_plantuml_code_blocks(markdown, &renderer, rel_image_url, true);

        // Now check the renderer was called with the correct arguments
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
            result.unwrap(),
            "Some text before\n\nFake renderer was here\nSome text after\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n"
        );
    }

    #[test]
    fn test_render_failure_with_fail_on_error_off() {
        let mut renderer = FakeRenderer::new();
        renderer.result = None; // Force an error
        let rel_image_url = "images/plantuml";

        // With silent_errors the image data is simply replaced with the error message
        let result =
            render_plantuml_code_blocks("```plantuml\nFoo\n```", &renderer, rel_image_url, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "FakeRenderer error");
    }

    #[test]
    fn test_render_failure_with_fail_on_error() {
        let mut renderer = FakeRenderer::new();
        renderer.result = None; // Force an error
        let rel_image_url = "images/plantuml";

        // With silent_errors the image data is simply replaced with the error message
        let result =
            render_plantuml_code_blocks("```plantuml\nFoo\n```", &renderer, rel_image_url, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "FakeRenderer error");
    }

    #[test]
    fn test_no_code_blocks() {
        let renderer = FakeRenderer::new();
        let rel_image_url = "images/plantuml";

        // With silent_errors the image data is simply replaced with the error message
        let result =
            render_plantuml_code_blocks("No code blocks here", &renderer, rel_image_url, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No code blocks here");
    }
}
