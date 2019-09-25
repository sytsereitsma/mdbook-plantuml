use pulldown_cmark::{Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::fmt::cmark;

pub trait PlantUMLCodeBlockRenderer {
    ///Renders the given code block and returns some HTML (compatible) code to replace
    ///the PlantUML code block
    fn render(&self, code_block: String) -> String;
}

/// Process all markdown 'events' detecting and transforming PlantUML code blocks
/// along the way.
pub fn render_plantuml_code_blocks(
    markdown: &str,
    processor: &impl PlantUMLCodeBlockRenderer,
) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    let mut in_plantuml_code_block = false;
    let mut plantuml_source = String::from("");

    let events = parser.map(|event| match event {
        Event::Start(Tag::CodeBlock(code)) => {
            //TODO: Nested code blocks are supported by commonmark
            //      How to deal with these?
            if code.clone().into_string() == "plantuml" {
                debug!("Started PlantUML code block");
                in_plantuml_code_block = true;
                Event::Text("".into())
            } else {
                Event::Start(Tag::CodeBlock(code))
            }
        }
        Event::Text(text) => {
            if in_plantuml_code_block {
                plantuml_source.push_str(&text.into_string());
                Event::Text("".into())
            } else {
                Event::Text(text)
            }
        }
        Event::End(Tag::CodeBlock(code)) => {
            if code.clone().into_string() == "plantuml" {
                in_plantuml_code_block = false;
                let plantuml_code = processor.render(plantuml_source.clone());
                plantuml_source = String::from("");

                Event::Text(plantuml_code.clone().into())
            } else {
                Event::End(Tag::CodeBlock(code))
            }
        }
        _ => event,
    });

    let mut markdown = String::with_capacity(markdown.len() + 1024);
    cmark(events, &mut markdown, None).unwrap();

    markdown
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use simulacrum::*;

    create_mock! {
        impl PlantUMLCodeBlockRenderer for RendererMock (self) {
            expect_render("render"):
                fn render(&self, code_block : String) -> String;
        }
    }

    #[test]
    fn no_code_blocks() {
        let mut mock_renderer = RendererMock::new();
        mock_renderer.expect_render().called_never();
        let result = render_plantuml_code_blocks(&String::from("#Some markdown"), &mock_renderer);
        assert_eq!("#Some markdown", result);
    }

    #[test]
    fn plantuml_code_block() {
        let plantuml_code = String::from("@startuml\n@enduml");
        let markdown = format!("#Some markdown\n```plantuml\n{}\n```", plantuml_code);
        let mut mock_renderer = RendererMock::new();
        mock_renderer
            .expect_render()
            .called_once()
            .returning(|_| String::from("froboz"));
        let result = render_plantuml_code_blocks(&markdown, &mock_renderer);
        assert_eq!("#Some markdown\n\nfroboz", result);
    }

    #[test]
    fn other_code_block() {
        //parsedown_cmark_to_cmark writes code block sections with four
        //consecutive backticks (which is completely legal), so we also provide
        //four backticks to make the comparison easier.
        let markdown = String::from("#Some markdown\n\n````mermaid\nbloob\n````");
        let mut mock_renderer = RendererMock::new();
        mock_renderer.expect_render().called_never();
        let result = render_plantuml_code_blocks(&markdown, &mock_renderer);
        assert_eq!(markdown, result);
    }
}
