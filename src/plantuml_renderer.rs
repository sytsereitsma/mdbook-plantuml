use crate::dir_cleaner::DirCleaner;
use crate::plantuml_backend::PlantUMLBackend;
use crate::plantuml_backend_factory;
use crate::plantumlconfig::PlantUMLConfig;
use sha1;
use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use base64::encode;

pub trait PlantUMLRendererTrait {
    fn render(&self, plantuml_code: &String, rel_img_url: &String, image_format: String) -> String;
}

/// Create the image names with the appropriate extension and path
/// The base name of the file is a SHA1 of the code block to avoid collisions
/// with existing and as a bonus prevent duplicate files.
pub fn get_image_filename(
    img_root: &PathBuf,
    plantuml_code: &String,
    image_format: &String,
) -> PathBuf {
    // See https://plantuml.com/command-line "Types of output files" for additional info
    let extension = {
        if plantuml_code.contains("@startditaa") {
            //ditaa only has png format support afaik
            String::from("png")
        } else if image_format == "" {
            String::from("svg")
        } else if image_format == "txt" {
            // -ttxt outputs an .atxt file
            String::from("atxt")
        } else if image_format == "braille" {
            // -tbraille outputs a .braille.png file
            String::from("braille.png")
        } else {
            image_format.to_string()
        }
    };

    let mut output_file = img_root.clone();
    output_file.push(sha1::Sha1::from(&plantuml_code).hexdigest());
    output_file.set_extension(extension);

    output_file
}

pub struct PlantUMLRenderer {
    backend: Box<dyn PlantUMLBackend>,
    cleaner: RefCell<DirCleaner>,
    img_root: PathBuf,
}

impl PlantUMLRenderer {
    pub fn new(cfg: &PlantUMLConfig, img_root: &PathBuf) -> PlantUMLRenderer {
        let renderer = PlantUMLRenderer {
            backend: plantuml_backend_factory::create(cfg),
            cleaner: RefCell::new(DirCleaner::new(img_root)),
            img_root: img_root.clone(),
        };

        renderer
    }

    fn create_image_datauri(image_path: &PathBuf) -> String {
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs#syntax

        let media_type = match image_path.extension().map(|s| s.to_str()).unwrap_or(Some("")) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("svg") => "image/svg+xml",
            Some("atxt") | Some("utxt") | Some("txt") => "text/plain",
            _ => "",
        };

        let mut image_file = fs::File::open(image_path).unwrap_or_else(|e| panic!("could not open file: {}", e));
        let mut image_bytes_buffer = Vec::new();
        image_file.read_to_end(&mut image_bytes_buffer).unwrap_or_else(|e| panic!("could not read file: {}", e));
        let encoded_value = encode(&image_bytes_buffer);

        format!("data:{};base64,{}", media_type, encoded_value)
    }

    fn create_image_datauri_element(image_path: &PathBuf) -> String {
        format!("<img src=\"{}\" />", PlantUMLRenderer::create_image_datauri(image_path))
    }

    pub fn render(
        &self,
        plantuml_code: &String,
        rel_img_url: &String,
        image_format: String,
    ) -> String {
        let output_file = get_image_filename(&self.img_root, plantuml_code, &image_format);
        if !output_file.exists() {
            if let Err(e) =
                self.backend
                    .render_from_string(plantuml_code, &image_format, &output_file)
            {
                error!("Failed to generate PlantUML diagram.");
                return String::from(format!("\nPlantUML rendering error:\n{}\n\n", e));
            }
        }

        self.cleaner.borrow_mut().keep(&output_file);
        PlantUMLRenderer::create_image_datauri_element(&output_file)
    }
}

impl PlantUMLRendererTrait for PlantUMLRenderer {
    fn render(&self, plantuml_code: &String, rel_img_url: &String, image_format: String) -> String {
        PlantUMLRenderer::render(self, plantuml_code, rel_img_url, image_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    struct BackendMock {
        is_ok: bool,
    }

    impl PlantUMLBackend for BackendMock {
        fn render_from_string(
            &self,
            plantuml_code: &String,
            image_format: &String,
            output_file: &PathBuf,
        ) -> Result<(), Error> {
            if self.is_ok {
                std::fs::write(output_file, format!("{}\n{}", plantuml_code, image_format))?;
                return Ok(());
            }
            bail!("Oh no")
        }
    }

    #[test]
    fn test_rendering() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(DirCleaner::new(&output_dir.path().to_path_buf())),
            img_root: PathBuf::from(output_dir.path().to_path_buf()),
        };

        let plantuml_code = String::from("some puml code");
        let code_hash = sha1::Sha1::from(&plantuml_code).hexdigest();

        // svg extension
        assert_eq!(
            String::from("<img src=\"data:image/svg+xml;base64,c29tZSBwdW1sIGNvZGUKc3Zn\" />"),
            renderer.render(
                &plantuml_code,
                &String::from("rel/url"),
                String::from("svg")
            )
        );

        // png extension
        assert_eq!(
            String::from("<img src=\"data:image/png;base64,c29tZSBwdW1sIGNvZGUKcG5n\" />"),
            renderer.render(
                &plantuml_code,
                &String::from("rel/url"),
                String::from("png")
            )
        );

        // txt extension
        assert_eq!(
            String::from("<img src=\"data:text/plain;base64,c29tZSBwdW1sIGNvZGUKdHh0\" />"),
            renderer.render(
                &plantuml_code,
                &String::from("rel/url"),
                String::from("txt")
            )
        );

        // utxt extension
        assert_eq!(
            String::from("<img src=\"data:text/plain;base64,c29tZSBwdW1sIGNvZGUKdHh0\" />"),
            renderer.render(
                &plantuml_code,
                &String::from("rel/url"),
                String::from("txt")
            )
        );
    }

    #[test]
    fn test_rendering_failure() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: false }),
            cleaner: RefCell::new(DirCleaner::new(&output_dir.path().to_path_buf())),
            img_root: PathBuf::from(output_dir.path().to_path_buf()),
        };

        assert_eq!(
            String::from("\nPlantUML rendering error:\nOh no\n\n"),
            renderer.render(
                &String::from(""),
                &String::from("rel/url"),
                String::from("svg")
            )
        );
    }

    #[test]
    fn test_get_image_filename_extension() {
        let get_extension_from_filename = |code: &String, img_format: String| -> String {
            let file_path = get_image_filename(&PathBuf::from("foo"), &code, &img_format)
                .to_string_lossy()
                .to_string();
            let firstdot = file_path.find('.').unwrap();
            file_path[firstdot + 1..].to_string()
        };

        assert_eq!(
            String::from("svg"),
            get_extension_from_filename(&String::from(""), String::from("svg"))
        );

        assert_eq!(
            String::from("eps"),
            get_extension_from_filename(&String::from(""), String::from("eps"))
        );

        assert_eq!(
            String::from("png"),
            get_extension_from_filename(&String::from(""), String::from("png"))
        );

        assert_eq!(
            String::from("svg"),
            get_extension_from_filename(&String::from(""), String::from(""))
        );

        assert_eq!(
            String::from("svg"),
            get_extension_from_filename(&String::from(""), String::from("svg"))
        );

        assert_eq!(
            String::from("atxt"),
            get_extension_from_filename(&String::from(""), String::from("txt"))
        );

        // Plantuml does this 'braille.png' extension
        assert_eq!(
            String::from("braille.png"),
            get_extension_from_filename(&String::from(""), String::from("braille"))
        );

        {
            //ditaa graphs
            // Note the format is overridden when rendering ditaa
            assert_eq!(
                String::from("png"),
                get_extension_from_filename(&String::from("@startditaa"), String::from("svg"))
            );

            assert_eq!(
                String::from("png"),
                get_extension_from_filename(&String::from("@startditaa"), String::from("png"))
            );

            assert_eq!(
                String::from("png"),
                get_extension_from_filename(
                    &String::from("Also when not at the start of the code block @startditaa"),
                    String::from("svg")
                )
            );
        }
    }

    #[test]
    fn test_get_image_filename() {
        let code = String::from("asgtfgl");
        let file_path = get_image_filename(&PathBuf::from("foo"), &code, &String::from("svg"));
        assert_eq!(PathBuf::from("foo"), file_path.parent().unwrap());
        assert_eq!(
            sha1::Sha1::from(&code).hexdigest(),
            file_path.file_stem().unwrap().to_str().unwrap()
        );
        assert_eq!(PathBuf::from("svg"), file_path.extension().unwrap());
    }
}
