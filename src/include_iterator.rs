/// Iterator to parse PlantUML content and yield included file names.
/// Block references are stripped from the filenames. So !include include.puml!block1 will
/// yield include.puml.
/// ```
pub struct IncludeIterator<'a> {
    lines: std::str::Lines<'a>,
}

impl<'a> IncludeIterator<'a> {
    /// Creates a new `IncludeIterator` from the given string slice.
    /// # Arguments
    /// * `content` - The string slice containing the content to iterate over.
    /// # Returns
    /// An `IncludeIterator` that yields lines from the provided content.
    pub fn new(content: &'a str) -> Self {
        IncludeIterator {
            lines: content.lines(),
        }
    }

    /// Extracts the filename or included part from a PlantUML `!include` directive.
    /// Handles different include variants such as `!include`, `!include_sub`, `!include_many`, and `!include_once`.
    /// Ignores stdlib sprite includes (enclosed in `<...>`).
    /// # Arguments
    /// * `trimmed_line` - The line to parse, expected to be trimmed of leading/trailing whitespace.
    /// # Returns
    /// An `Option<&str>` containing the filename or included part if parsing is successful, or `None` if the line is not a valid include directive.
    fn get_include_file_part(trimmed_line: &str) -> Option<&str> {
        match Self::trim_include_directive(trimmed_line) {
            Some(file_part) => {
                if file_part.is_empty() {
                    log::warn!(
                        "Malformed !include directive, missing filename: {}",
                        trimmed_line
                    );

                    None
                } else if file_part.starts_with('<') {
                    // This is a an standard library sprite include, there is no file associated with it
                    None
                } else {
                    Some(file_part)
                }
            }
            None => None,
        }
    }

    /// Trims the `!include` directive and its variants from the start of a line (e.g. `!include file.puml!block` -> `file.puml!block`, `!include <C4/C8>` -> `<C4/C8>`).
    /// # Arguments
    /// * `trimmed_line` - The line to parse, expected to be trimmed of leading/trailing whitespace.
    /// # Returns
    /// An `Option<&str>` containing the remaining part of the line after the `!include` directive, or `None` if the line does not start with an include directive.
    fn trim_include_directive(trimmed_line: &str) -> Option<&str> {
        if !trimmed_line.starts_with("!include") {
            return None;
        }

        // We have an include, shift the slice to the end of the !include keyword to test for the different include types
        let variant_start = &trimmed_line["!include".len()..];

        // Now find the (potential) start of the filename
        let include_end = if variant_start.starts_with("sub") {
            "sub".len()
        } else if variant_start.starts_with("_many") {
            "_many".len()
        } else if variant_start.starts_with("_once") {
            "_once".len()
        } else {
            0usize
        };

        Some(variant_start[include_end..].trim_start())
    }

    /// Trims comments and surrounding whitespace from a line (e.g., `  !include file.puml   ' comment` -> `!include file.puml`).
    /// Removes everything after the first occurrence of `/'` or `'`, which are used as comment markers in PlantUML.
    /// # Arguments
    /// * `line` - The line to trim.
    /// # Returns
    /// A string slice with comments and surrounding whitespace removed.
    fn trim_comments_and_spaces(line: &str) -> &str {
        let trimmed_comment = if let Some(index) = line.find("/'") {
            &line[..index]
        } else if let Some(index) = line.find('\'') {
            &line[..index]
        } else {
            line
        };

        trimmed_comment.trim()
    }

    /// Trims any block specifier from an include name (e.g., `file.puml!block` -> `file.puml`).
    /// Removes everything after the first `!` character, which may be used to specify a block in an include directive.
    /// # Arguments
    /// * `include_name` - The include name to trim.
    /// # Returns
    /// A string slice with the block specifier removed, if present.
    fn trim_block(include_name: &str) -> &str {
        if let Some(block_index) = include_name.find('!') {
            include_name[..block_index].trim()
        } else {
            include_name
        }
    }
}

impl<'a> Iterator for IncludeIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        for line in self.lines.by_ref() {
            let trimmed_line = Self::trim_comments_and_spaces(line);

            let include_name_part = Self::get_include_file_part(trimmed_line);
            if include_name_part.is_none() {
                continue;
            }

            // "file.puml!block" -> "file.puml"
            let include_name = Self::trim_block(include_name_part.unwrap());

            if include_name.is_empty() {
                continue;
            }

            return Some(include_name);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_iterator() {
        let content = r#"
        Some text
        !include diagram1.puml ' This is a comment
        More text
        !include <diagram2.puml> /' Stdlib sprite includes are ignored
        !includesub   spaces_before_and_after.puml   
        !includesub   with_block_and_spaces_after.puml!block   
        The following include misses a closing angle bracket >
        !include <malformed_include2.puml
        The following includes miss their filename
        !include
        !includesub
        !include !block
        !includesub !block
        !include_once there_can_be_only_once.puml
        !include_many many_files.puml
        "#;

        let mut iterator = IncludeIterator::new(content);

        assert_eq!(iterator.next(), Some("diagram1.puml"));
        assert_eq!(iterator.next(), Some("spaces_before_and_after.puml"));
        assert_eq!(iterator.next(), Some("with_block_and_spaces_after.puml"));
        assert_eq!(iterator.next(), Some("there_can_be_only_once.puml"));
        assert_eq!(iterator.next(), Some("many_files.puml"));
        assert_eq!(iterator.next(), None); // Malformed include should be skipped
    }

    #[test]
    fn test_trim_comments_and_spaces() {
        assert_eq!(
            IncludeIterator::trim_comments_and_spaces(
                "   !include diagram1.puml ' This is a comment   "
            ),
            "!include diagram1.puml"
        );
        assert_eq!(
            IncludeIterator::trim_comments_and_spaces(
                "   !include diagram1.puml /' This is a block comment   "
            ),
            "!include diagram1.puml"
        );
        assert_eq!(
            IncludeIterator::trim_comments_and_spaces("   !include diagram1.puml "),
            "!include diagram1.puml"
        );
        assert_eq!(IncludeIterator::trim_comments_and_spaces("  "), "");
        assert_eq!(IncludeIterator::trim_comments_and_spaces("  text"), "text");
        assert_eq!(IncludeIterator::trim_comments_and_spaces("text "), "text");
        assert_eq!(IncludeIterator::trim_comments_and_spaces(" ' "), "");
        assert_eq!(IncludeIterator::trim_comments_and_spaces(" /' "), "");
        assert_eq!(IncludeIterator::trim_comments_and_spaces(""), "");
    }

    #[test]
    fn test_get_include_file_part() {
        assert_eq!(
            IncludeIterator::get_include_file_part("!include diagram1.puml"),
            Some("diagram1.puml")
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!includesub diagram2.puml"),
            Some("diagram2.puml")
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!include_many diagram3.puml"),
            Some("diagram3.puml")
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!include_once diagram4.puml"),
            Some("diagram4.puml")
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("Some other text"),
            None
        );
        assert_eq!(IncludeIterator::get_include_file_part(""), None);

        // Missing filename
        assert_eq!(IncludeIterator::get_include_file_part("!include"), None);
        assert_eq!(IncludeIterator::get_include_file_part("!includesub"), None);
        assert_eq!(
            IncludeIterator::get_include_file_part("!include_many"),
            None
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!include_once"),
            None
        );

        // Stdlib sprites are ignored
        assert_eq!(
            IncludeIterator::get_include_file_part("!include <sprite.puml>"),
            None
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!includesub <sprite.puml>"),
            None
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!include_many <sprite.puml>"),
            None
        );
        assert_eq!(
            IncludeIterator::get_include_file_part("!include_once <sprite.puml>"),
            None
        );
    }
}
