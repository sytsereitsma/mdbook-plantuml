use std::collections::HashMap;

/// Holds the parsed code block info string
pub struct InfoString<'a> {
    /// Raw info string
    info_string: &'a str,
    /// Code block language
    language: Option<&'a str>,
    /// Config options. A comma separated list of <key>=<value> pairs (mdbook-plantuml specific)
    /// The value is optional. (example "```plantuml,format=png,foo=bar")
    config: HashMap<&'a str, Option<&'a str>>,
}

impl<'a> InfoString<'a> {
    fn from(raw_info_string: &'a str) -> Self {
        let mut created = Self {
            info_string: raw_info_string.trim(),
            language: None,
            config: HashMap::new(),
        };

        // Little helper to parse a key-vaue pair. Returns None if part is None, or an empty string when trimmed
        let parse_value = |part: Option<&'a str>| {
            if let Some(value) = part.and_then(|k| Some(k.trim())) {
                if !value.is_empty() {
                    return Some(value);
                }
            }

            None
        };

        // Extract the config key value pairs.
        // First element in the list is always the language (unless the first list item contains a '=')
        // info string separator char was ',', but commonmark specifeis this should be a space. Left ','
        // separator support for BW compatibility
        let mut check_language = true;
        for part in created.info_string.split(&[',', ' ']) {
            let part = part.trim();

            // first element is language if it does not contain a '='
            if check_language && !part.is_empty() && !part.contains('=') {
                created.language = Some(part);
            } else {
                // Get and parse the key value pairs
                let mut config = part.split('=');
                if let Some(cfg_key) = parse_value(config.next()) {
                    if let Some(cfg_value) = parse_value(config.next()) {
                        created.config.insert(cfg_key, Some(cfg_value));
                    } else {
                        created.config.insert(cfg_key, None);
                    }
                }
            }

            check_language = false;
        }

        created
    }

    /// Returns true if this code block is plantuml (i.e. starts with plantuml or puml)
    fn is_plantuml(&self) -> bool {
        self.language == Some("plantuml") || self.language == Some("puml")
    }
}

/// Code block representation
pub struct CodeBlock<'a> {
    // Full block, including opening and closing fences
    full_block: &'a str,
    // The code block's info string
    info_string: InfoString<'a>,
    // The code block's code, stripped from opening and closing fences
    code: &'a str,
}

impl<'a> CodeBlock<'a> {
    /// Returns the image format (file extension) PlantUML needs to generate for this code block
    fn get_image_format(&self) -> &'a str {
        if self.code.contains("@startditaa") {
            // Ditaa only supports png
            return "png";
        } else if let Some(format) = self
            .info_string
            .config
            .get("format")
            .and_then(|fmt| fmt.as_ref())
        {
            // User specified image format (e.g. png for "```plantuml,format=png")
            return format;
        }

        // Default to svg
        "svg"
    }
}

/// Text block representation
pub struct TextBlock<'a> {
    /// The raw text in the text block
    text: &'a str,
}

/// The markdown block type
pub enum Block<'a> {
    /// A markdown code block
    Code(CodeBlock<'a>),
    /// A markdown 'text' block (i.e. everything but code blocks)
    Text(TextBlock<'a>),
}

/// A code fence
#[derive(Debug, PartialEq, Eq)]
struct CodeFence {
    /// The code fence character
    fence_char: char,
    /// The code fence's width (e.g. 3 for "```")
    width: usize,
}

/// Implements an interator over a markdown document
/// The markdown document is split into `Block::Text` and `Block::Code` elements
pub struct MarkdownIterator<'a> {
    markdown: &'a str,
    lines_it: std::iter::Peekable<std::str::Lines<'a>>,
}

impl<'a> MarkdownIterator<'a> {
    /// Construct a new markdown iterator from the given markdown source
    pub fn new(markdown: &'a str) -> MarkdownIterator {
        MarkdownIterator {
            markdown,
            lines_it: markdown.lines().peekable(),
        }
    }

    /// Is the given fence an opening fence for a oneliner (e.g. "```oneline``` foo the bar")
    fn is_oneline_fence(line: &'a str, opening_fence: &CodeFence) -> bool {
        if opening_fence.fence_char == '~' {
            // Info strings for tilde code blocks can contain backticks and tildes.
            // So ~ fences cannot be used for one-liners
            return false;
        }

        // Could be an indented fence, so trim start
        let rest_of_line = &line.trim_start()[opening_fence.width..];

        let mut width: usize = 0;

        for c in rest_of_line.chars() {
            if c == opening_fence.fence_char {
                width += 1;
            } else if width >= opening_fence.width {
                // Closing fence found
                break;
            } else {
                // Restart the find
                width = 0;
            }
        }

        return width == opening_fence.width;
    }

    /// Returns a CodeFence when the line starts with a valid/expected opening or closing code fence
    fn get_code_fence(line: &'a str, fence_to_match: Option<&CodeFence>) -> Option<CodeFence> {
        let mut width: usize = 0;
        let mut fence_char: char = 'X';
        let mut column = 1; // 1 based

        for c in line.chars() {
            if c == '`' || c == '~' {
                if fence_char == 'X' {
                    fence_char = c;
                    width += 1;
                } else if fence_char == c {
                    width += 1;
                } else {
                    break;
                }
            } else if c != ' ' || column >= 4 {
                // More than 3 leading spaces, or a non space character
                break;
            }

            column += 1;
        }

        let fence = CodeFence { fence_char, width };
        if fence_char == 'X' || width <= 2 {
            return None;
        }

        match fence_to_match {
            Some(opening_fence) => {
                if opening_fence.fence_char != fence_char {
                    // Closing fence needs to use the same fence char
                    return None;
                } else if opening_fence.width > width {
                    // Closing fence needs to be at least as wide as opening fence
                    return None;
                } else if line.trim().len() > width {
                    // We've found a closing fence with text behind it, this is not considered a closing fence
                    return None;
                }
            }
            None => {
                if Self::is_oneline_fence(&line, &fence) {
                    // We've found an opening fence, but it's a oneliner
                    return None;
                }
            }
        }

        Some(fence)
    }

    fn process_code_block(&mut self, fence_line: &'a str, fence: CodeFence) -> Option<Block<'a>> {
        if self.lines_it.peek().is_none() {
            return Some(Block::Code(CodeBlock {
                full_block: fence_line,
                info_string: InfoString::from(&fence_line[fence.width..]),
                code: &fence_line[fence_line.len() - 1..fence_line.len() - 1],
            }));
        }

        let start_of_code = self.lines_it.next().unwrap();
        let mut closing_fence_line: Option<&'a str> = None;

        while let Some(line) = self.lines_it.next() {
            if Self::get_code_fence(line, Some(&fence)).is_some() {
                closing_fence_line = Some(line);
                break;
            }
        }

        let start_code_idx = start_of_code.as_ptr() as usize - self.markdown.as_ptr() as usize;
        let start_full_block_idx = fence_line.as_ptr() as usize - self.markdown.as_ptr() as usize;
        let end_code_idx;
        let end_full_block_idx;

        match closing_fence_line {
            Some(end_of_code) => {
                end_code_idx = end_of_code.as_ptr() as usize - self.markdown.as_ptr() as usize;
                if let Some(next_line) = self.lines_it.peek() {
                    end_full_block_idx =
                        next_line.as_ptr() as usize - self.markdown.as_ptr() as usize;
                } else {
                    end_full_block_idx = (end_of_code.as_ptr() as usize
                        - self.markdown.as_ptr() as usize)
                        + end_of_code.len();
                }
            }
            None => {                
                end_code_idx = self.markdown.len();
                end_full_block_idx = self.markdown.len();
            }
        }

        Some(Block::Code(CodeBlock {
            full_block: &self.markdown[start_full_block_idx..end_full_block_idx],
            info_string: InfoString::from(&fence_line[fence.width..]),
            code: &self.markdown[start_code_idx..end_code_idx],
        }))
    }

    fn process_text_block(&mut self, start_line: &'a str) -> Option<Block<'a>> {
        let mut code_fence_line: Option<&'a str> = None;

        // Use peek, because when we encounter a code fence it is needed for the code block returned by the next() call.
        while let Some(line) = self.lines_it.peek() {
            if Self::get_code_fence(line, None).is_some() {
                code_fence_line = Some(line);
                break;
            } else {
                // Eat it
                self.lines_it.next();
            }
        }

        let start_text_idx = start_line.as_ptr() as usize - self.markdown.as_ptr() as usize;
        let end_text_idx = if let Some(end_line) = code_fence_line {
            // Start of line, we only want the closing newline, not the fence
            end_line.as_ptr() as usize - self.markdown.as_ptr() as usize
        } else {
            self.markdown.len()
        };

        Some(Block::Text(TextBlock {
            text: &self.markdown[start_text_idx..end_text_idx],
        }))
    }
}

impl<'a> Iterator for MarkdownIterator<'a> {
    type Item = Block<'a>;
    fn next(&mut self) -> Option<Block<'a>> {
        if let Some(line) = self.lines_it.next() {
            if let Some(fence) = Self::get_code_fence(line, None) {
                return self.process_code_block(line, fence);
            } else {
                return self.process_text_block(line);
            }
        }

        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    impl CodeFence {
        fn new(fence_char: char, width: usize) -> CodeFence {
            CodeFence { fence_char, width }
        }
    }

    macro_rules! assert_text_block {
        ($expected_text:expr, $block_opt:expr) => {{
            if let Some(block) = $block_opt {
                match block {
                    Block::Text(tb) => assert_eq!($expected_text, tb.text),
                    _ => assert!(false, "expected Block::Text"),
                }
            } else {
                assert!(false, "Block is None");
            }
        }};
    }

    macro_rules! assert_code_block {
        ($expected_full_block:expr, $expected_code:expr, $block_opt:expr) => {{
            if let Some(block) = $block_opt {
                match block {
                    Block::Code(cb) => {
                        assert_eq!($expected_code, cb.code, "Block code");
                        assert_eq!($expected_full_block, cb.full_block, "Full block");
                    },
                    _ => assert!(false, "expected Block::Code"),
                }
            } else {
                assert!(false, "Block is None");
            }
        }};
    }

    #[test]
    fn iterate_returns_none_for_empty_markdown() {
        let mut mit = MarkdownIterator::new("");
        assert!(mit.next().is_none());
    }

    #[test]
    fn iterate_returns_text_block_for_simple_text() {
        let mut mit = MarkdownIterator::new("Foo");
        assert_text_block!("Foo", mit.next());
    }

    #[test]
    fn iterate_returns_text_block_for_multiline_text() {
        let mut mit = MarkdownIterator::new("Waldorf\nStentor");
        assert_text_block!("Waldorf\nStentor", mit.next());
    }

    #[test]
    fn iterate_returns_code_block() {
        let mut mit = MarkdownIterator::new("```\nCow\n```");
        assert_code_block!("```\nCow\n```", "Cow\n", mit.next());
    }

    #[test]
    fn iterate_returns_open_ended_code_block() {
        let mut mit = MarkdownIterator::new("```\nCow\n");
        assert_code_block!("```\nCow\n", "Cow\n", mit.next());
    }

    #[test]
    fn iterate_returns_open_ended_empty_code_block() {
        let mut mit = MarkdownIterator::new("```\n");
        assert_code_block!("```\n", "", mit.next());
    }

    #[test]
    fn iterate_multiple_block_types() {
        let mut mit = MarkdownIterator::new("Waldorf\n```\nfoo\n```\nStentor");
        assert_text_block!("Waldorf\n", mit.next());
        assert_code_block!("```\nfoo\n```\n", "foo\n", mit.next());
        assert_text_block!("Stentor", mit.next());
        assert!(mit.next().is_none());
    }

    #[test]
    fn iterate_returns_nested_code_block() {
        let mut mit = MarkdownIterator::new("````\nCow\n```Chicken\n```\n````");
        assert_code_block!("````\nCow\n```Chicken\n```\n````", "Cow\n```Chicken\n```\n", mit.next());
    }

    #[test]
    fn iterate_oneline_block_is_text() {
        let mut mit = MarkdownIterator::new("```oneliner```");
        assert_text_block!("```oneliner```", mit.next());

        // A more complex one, with a false end fence
        let mut mit = MarkdownIterator::new("````oneliner```blorgh````ff");
        assert_text_block!("````oneliner```blorgh````ff", mit.next());
    }

    #[test]
    fn ignore_closing_fences_with_info_string() {
        let mut mit = MarkdownIterator::new("```\na\n``` info\nb\n```");
        assert_code_block!("```\na\n``` info\nb\n```", "a\n``` info\nb\n", mit.next());
    }

    #[test]
    fn get_code_fence() {
        assert_eq!(None, MarkdownIterator::get_code_fence("", None));
        assert_eq!(None, MarkdownIterator::get_code_fence("Staple", None));
        // More than 3 leading spaces -> Not a fence
        assert_eq!(None, MarkdownIterator::get_code_fence("    ```", None));
        assert_eq!(
            None,
            MarkdownIterator::get_code_fence("    ~~~", Some(&CodeFence::new('~', 3)))
        );

        // Up to 3 leading spaces -> it's a fence
        assert_eq!(
            Some(CodeFence::new('~', 3)),
            MarkdownIterator::get_code_fence("   ~~~", None)
        );

        // Spaces rule also applies to closing fences
        assert_eq!(
            Some(CodeFence::new('~', 3)),
            MarkdownIterator::get_code_fence("   ~~~", Some(&CodeFence::new('~', 3)))
        );

        // Check end fence is matching
        assert_eq!(
            None,
            MarkdownIterator::get_code_fence("~~~", Some(&CodeFence::new('~', 4)))
        );
        // Closing fence must be at least as wide
        assert_eq!(
            Some(CodeFence::new('~', 5)),
            MarkdownIterator::get_code_fence("~~~~~", Some(&CodeFence::new('~', 4)))
        );
        // Closing fence cannot contain extra characters behind it
        assert_eq!(
            None,
            MarkdownIterator::get_code_fence("~~~~~ a", Some(&CodeFence::new('~', 4)))
        );
        // Only whitespace after the closing fence is not considered text -> valid closing fence
        assert_eq!(
            Some(CodeFence::new('~', 5)),
            MarkdownIterator::get_code_fence("~~~~~   ", Some(&CodeFence::new('~', 4)))
        );
        assert_eq!(
            None,
            MarkdownIterator::get_code_fence("````", Some(&CodeFence::new('~', 4)))
        );
        assert_eq!(
            None,
            MarkdownIterator::get_code_fence("~~~~", Some(&CodeFence::new('`', 4)))
        );
        assert_eq!(
            Some(CodeFence::new('~', 4)),
            MarkdownIterator::get_code_fence("~~~~", Some(&CodeFence::new('~', 4)))
        );
        assert_eq!(
            Some(CodeFence::new('`', 4)),
            MarkdownIterator::get_code_fence("````", Some(&CodeFence::new('`', 4)))
        );

        // Need at least 3 consecutive identical fence chars
        assert_eq!(None, MarkdownIterator::get_code_fence("``", None));
        assert_eq!(None, MarkdownIterator::get_code_fence("~~", None));
        assert_eq!(None, MarkdownIterator::get_code_fence("``~`", None));

        assert_eq!(
            Some(CodeFence::new('`', 3)),
            MarkdownIterator::get_code_fence("```", None)
        );
        assert_eq!(
            Some(CodeFence::new('`', 4)),
            MarkdownIterator::get_code_fence("````", None)
        );

        assert_eq!(
            Some(CodeFence::new('~', 3)),
            MarkdownIterator::get_code_fence("~~~", None)
        );
        assert_eq!(
            Some(CodeFence::new('~', 4)),
            MarkdownIterator::get_code_fence("~~~~", None)
        );

        assert_eq!(
            Some(CodeFence::new('~', 3)),
            MarkdownIterator::get_code_fence("~~~", None)
        );
    }

    #[test]
    fn is_oneline_fence() {
        assert!(MarkdownIterator::is_oneline_fence(
            "```oneline```",
            &CodeFence::new('`', 3)
        ));
        assert!(MarkdownIterator::is_oneline_fence(
            "```oneline```abc",
            &CodeFence::new('`', 3)
        ));
        assert!(!MarkdownIterator::is_oneline_fence(
            "```oneline``abc",
            &CodeFence::new('`', 3)
        ));
        assert!(!MarkdownIterator::is_oneline_fence(
            "```oneline",
            &CodeFence::new('`', 3)
        ));
        assert!(!MarkdownIterator::is_oneline_fence(
            "```",
            &CodeFence::new('`', 3)
        ));

        assert!(MarkdownIterator::is_oneline_fence(
            "```oneline``abc```def",
            &CodeFence::new('`', 3)
        ));
        assert!(!MarkdownIterator::is_oneline_fence(
            "```oneline````def",
            &CodeFence::new('`', 3)
        ));
        assert!(!MarkdownIterator::is_oneline_fence(
            "```",
            &CodeFence::new('`', 3)
        ));

        // Closing fence should be exactly as wide
        assert!(!MarkdownIterator::is_oneline_fence(
            "``` aabc `````",
            &CodeFence::new('`', 3)
        ));

        // Indented fence
        assert!(MarkdownIterator::is_oneline_fence(
            "   ``` abc ```",
            &CodeFence::new('`', 3)
        ));
        assert!(!MarkdownIterator::is_oneline_fence(
            "   ``` abc",
            &CodeFence::new('`', 3)
        ));

        assert!(!MarkdownIterator::is_oneline_fence(
            "~~~ foo bar ``` ~~~",
            &CodeFence::new('~', 3)
        ));
    }

    #[test]
    fn test_infostring_plantuml_detection() {
        assert!(InfoString::from("plantuml").is_plantuml());
        assert!(InfoString::from("puml").is_plantuml());
        assert!(InfoString::from("plantuml,format=svg").is_plantuml());
        assert!(InfoString::from("puml,format=svg").is_plantuml());

        assert!(!InfoString::from(",plantuml").is_plantuml()); // Bogus info string
        assert!(!InfoString::from("plantUML").is_plantuml()); // Case sensitive
        assert!(!InfoString::from("c++").is_plantuml());
    }

    #[test]
    fn test_infostring_config_parsing() {
        let info = InfoString::from("");
        assert!(info.language.is_none());
        assert!(info.config.is_empty());
        assert_eq!(info.info_string, "");

        let info = InfoString::from("abc");
        assert_eq!(info.language, Some("abc"));
        assert!(info.config.is_empty());
        assert_eq!(info.info_string, "abc");

        let info = InfoString::from("abc=def");
        assert!(info.language.is_none());
        assert_eq!(info.config, HashMap::from([("abc", Some("def"))]));
        assert_eq!(info.info_string, "abc=def");

        let info = InfoString::from("abc=");
        assert!(info.language.is_none());
        assert_eq!(info.config, HashMap::from([("abc", None)]));
        assert_eq!(info.info_string, "abc=");

        let info = InfoString::from("c++,abc=");
        assert_eq!(info.language, Some("c++"));
        assert_eq!(info.config, HashMap::from([("abc", None)]));
        assert_eq!(info.info_string, "c++,abc=");

        let info = InfoString::from("rs,abc=,qq,def=12");
        assert_eq!(info.language, Some("rs"));
        assert_eq!(
            info.config,
            HashMap::from([("abc", None), ("qq", None), ("def", Some("12"))])
        );
        assert_eq!(info.info_string, "rs,abc=,qq,def=12");

        let info = InfoString::from("rs abc= qq def=12");
        assert_eq!(info.language, Some("rs"));
        assert_eq!(
            info.config,
            HashMap::from([("abc", None), ("qq", None), ("def", Some("12"))])
        );
        assert_eq!(info.info_string, "rs abc= qq def=12");
    }

    #[test]
    fn test_plantuml_codeblock_format_detection() {
        macro_rules! get_image_format {
            ($info_str:expr) => {{
                get_image_format!($info_str, "foo")
            }};
            ($info_str:expr, $code: expr) => {{
                let code_block = CodeBlock {
                    full_block: "",
                    code: $code,
                    info_string: InfoString::from($info_str),
                };

                code_block.get_image_format()
            }};
        }

        assert_eq!("svg", get_image_format!("plantuml"));
        assert_eq!("svg", get_image_format!("plantuml,format=svg"));
        assert_eq!("png", get_image_format!("plantuml,format=png"));
        assert_eq!("txt", get_image_format!("plantuml,bruh=123,format=txt"));
        assert_eq!(
            "jpg",
            get_image_format!("plantuml,bruh=123,format=jpg,bruh=123")
        );
        assert_eq!("png", get_image_format!("plantuml", "@startditaa"));

        // Error/edge cases
        assert_eq!("svg", get_image_format!("plantuml,format="));
        assert_eq!("svg", get_image_format!("plantuml,format"));
        assert_eq!(
            "svg",
            get_image_format!("plantuml,bruh=123,format=,bruh=123")
        );
        assert_eq!("svg", get_image_format!("plantuml,bruh=123"));
    }
}