use failure::Error;
use sha1;
use std::path::PathBuf;

pub trait PlantUMLBackend {
    ///Render a PlantUML string to file and return the diagram URL path to this
    ///file (as a String) for use in a link.
    /// # Arguments
    /// * `plantuml_code` - The present source of the code block, if it does not match with the cached code block None is returned
    fn render_from_string(&self, plantuml_code: &String) -> Result<PathBuf, Error>;
}

// Helper to easily get the extension from a PathBuf that is known to have an
// extension
pub fn get_extension(filename: &PathBuf) -> String {
    filename.extension().unwrap().to_string_lossy().to_string()
}

/// Create the image names with the appropriate extension and path
/// The base name of the file is a SHA1 of the code block to avoid collisions
/// with existing and as a bonus prevent duplicate files.
pub fn get_image_filename(img_root: &PathBuf, plantuml_code: &String) -> PathBuf {
    let extension = {
        if plantuml_code.contains("@startditaa") {
            String::from("png")
        } else {
            String::from("svg")
        }
    };

    let mut output_file = img_root.clone();
    output_file.push(sha1::Sha1::from(&plantuml_code).hexdigest());
    output_file.set_extension(extension);

    output_file
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_extension() {
        let get_extension_for_code = |code: &String| -> String {
            let file_path = get_image_filename(&PathBuf::from("foo"), &code);
            get_extension(&file_path)
        };

        assert_eq!(
            String::from("svg"),
            get_extension_for_code(&String::from("C --|> D"))
        );

        assert_eq!(
            String::from("png"),
            get_extension_for_code(&String::from("@startditaa"))
        );

        assert_eq!(
            String::from("png"),
            get_extension_for_code(&String::from(
                "Also when not at the start of the code block @startditaa"
            ))
        );
    }

    #[test]
    fn test_get_image_filename() {
        let code = String::from("asgtfgl");
        let file_path = get_image_filename(&PathBuf::from("foo"), &code);
        assert_eq!(PathBuf::from("foo"), file_path.parent().unwrap());
        assert_eq!(
            sha1::Sha1::from(&code).hexdigest(),
            file_path.file_stem().unwrap().to_str().unwrap()
        );
        assert_eq!(PathBuf::from("svg"), file_path.extension().unwrap());
    }
}
