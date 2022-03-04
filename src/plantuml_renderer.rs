use crate::dir_cleaner::DirCleaner;
use crate::plantuml_backend::PlantUMLBackend;
use crate::plantuml_backend_factory;
use crate::plantumlconfig::PlantUMLConfig;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

pub trait PlantUMLRendererTrait {
    fn render(&self, plantuml_code: &str, rel_img_url: &str, image_format: String) -> String;
}

/// Create the image names with the appropriate extension and path
/// The base name of the file is a SHA1 of the code block to avoid collisions
/// with existing and as a bonus prevent duplicate files.
pub fn get_image_filename(img_root: &Path, plantuml_code: &str, image_format: &str) -> PathBuf {
    // See https://plantuml.com/command-line "Types of output files" for additional info
    let extension = {
        if plantuml_code.contains("@startditaa") {
            // ditaa only has png format support afaik
            "png"
        } else if image_format.is_empty() {
            "svg"
        } else if image_format == "txt" {
            // -ttxt outputs an .atxt file
            "atxt"
        } else if image_format == "braille" {
            // -tbraille outputs a .braille.png file
            "braille.png"
        } else {
            image_format
        }
    };

    let mut output_file = img_root.to_path_buf();
    output_file.push(sha1::Sha1::from(&plantuml_code).hexdigest());
    output_file.set_extension(extension);

    output_file
}

pub struct PlantUMLRenderer {
    backend: Box<dyn PlantUMLBackend>,
    cleaner: RefCell<DirCleaner>,
    img_root: PathBuf,
    clickable_img: bool,
}

impl PlantUMLRenderer {
    pub fn new(cfg: &PlantUMLConfig, img_root: &Path) -> Self {
        let renderer = Self {
            backend: plantuml_backend_factory::create(cfg),
            cleaner: RefCell::new(DirCleaner::new(img_root)),
            img_root: img_root.to_path_buf(),
            clickable_img: cfg.clickable_img,
        };

        renderer
    }

    fn create_md_link(rel_img_url: &str, image_path: &Path, clickable: bool) -> String {
        let img_url = format!(
            "{}/{}",
            rel_img_url,
            image_path.file_name().unwrap().to_str().unwrap()
        );
        if clickable {
            format!("[![]({})]({})\n\n", img_url, img_url)
        } else {
            format!("![]({})\n\n", img_url)
        }
    }

    fn create_inline_image(image_path: &Path) -> String {
        log::debug!("Creating inline image from {:?}", image_path);
        let raw_source = fs::read(image_path).unwrap();
        let txt = unsafe { String::from_utf8_unchecked(raw_source) };
        format!("\n```txt\n{}```\n", txt)
    }

    pub fn render(&self, plantuml_code: &str, rel_img_url: &str, image_format: &str) -> String {
        let output_file = get_image_filename(&self.img_root, plantuml_code, image_format);
        if !output_file.exists() {
            if let Err(e) =
                self.backend
                    .render_from_string(plantuml_code, image_format, &output_file)
            {
                log::error!("Failed to generate PlantUML diagram.");
                return format!("\nPlantUML rendering error:\n{}\n\n", e);
            }
        }

        self.cleaner.borrow_mut().keep(&output_file);
        let extension = output_file.extension().unwrap_or_default();
        if extension == "atxt" || extension == "utxt" {
            Self::create_inline_image(&output_file)
        } else {
            Self::create_md_link(rel_img_url, &output_file, self.clickable_img)
        }
    }
}

impl PlantUMLRendererTrait for PlantUMLRenderer {
    fn render(&self, plantuml_code: &str, rel_img_url: &str, image_format: String) -> String {
        Self::render(self, plantuml_code, rel_img_url, &image_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::{bail, Error};
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_create_md_link() {
        assert_eq!(
            String::from("![](foo/bar/baz.svg)\n\n"),
            PlantUMLRenderer::create_md_link("foo/bar", Path::new("/froboz/baz.svg"), false)
        );

        assert_eq!(
            "![](/baz.svg)\n\n",
            PlantUMLRenderer::create_md_link("", Path::new("baz.svg"), false)
        );

        assert_eq!(
            String::from("![](/baz.svg)\n\n"),
            PlantUMLRenderer::create_md_link("", Path::new("foo/baz.svg"), false)
        );
    }

    struct BackendMock {
        is_ok: bool,
    }

    impl PlantUMLBackend for BackendMock {
        fn render_from_string(
            &self,
            plantuml_code: &str,
            image_format: &str,
            output_file: &Path,
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
            cleaner: RefCell::new(DirCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
        };

        let plantuml_code = "some puml code";
        let code_hash = sha1::Sha1::from(&plantuml_code).hexdigest();

        assert_eq!(
            format!("![](rel/url/{}.svg)\n\n", code_hash),
            renderer.render(plantuml_code, "rel/url", "svg",)
        );

        // png extension
        assert_eq!(
            format!("![](rel/url/{}.png)\n\n", code_hash),
            renderer.render(plantuml_code, "rel/url", "png",)
        );

        // txt extension
        assert_eq!(
            format!("\n```txt\n{}\ntxt```\n", plantuml_code), /* image format is appended by
                                                               * fake backend */
            renderer.render(plantuml_code, "rel/url", "txt",)
        );

        // utxt extension
        assert_eq!(
            format!("\n```txt\n{}\ntxt```\n", plantuml_code), /* image format is appended by
                                                               * fake backend */
            renderer.render(plantuml_code, "rel/url", "txt",)
        );
    }

    #[test]
    fn test_rendering_clickable() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(DirCleaner::new(&output_dir.path().to_path_buf())),
            img_root: PathBuf::from(output_dir.path().to_path_buf()),
            clickable_img: true,
        };

        let plantuml_code = String::from("some puml code");
        let code_hash = sha1::Sha1::from(&plantuml_code).hexdigest();
        assert_eq!(
            format!(
                "[![](rel/url/{}.svg)](rel/url/{}.svg)\n\n",
                code_hash, code_hash
            ),
            renderer.render(&plantuml_code, &String::from("rel/url"), "svg")
        );
    }

    #[test]
    fn test_rendering_failure() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: false }),
            cleaner: RefCell::new(DirCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
        };

        assert_eq!(
            String::from("\nPlantUML rendering error:\nOh no\n\n"),
            renderer.render("", "rel/url", "svg",)
        );
    }

    #[test]
    fn test_get_image_filename_extension() {
        let get_extension_from_filename = |code: &str, img_format: &str| -> String {
            let file_path = get_image_filename(Path::new("foo"), code, img_format)
                .to_string_lossy()
                .to_string();
            let firstdot = file_path.find('.').unwrap();
            file_path[firstdot + 1..].to_string()
        };

        assert_eq!(String::from("svg"), get_extension_from_filename("", "svg"));

        assert_eq!(String::from("eps"), get_extension_from_filename("", "eps"));

        assert_eq!(String::from("png"), get_extension_from_filename("", "png"));

        assert_eq!(String::from("svg"), get_extension_from_filename("", ""));

        assert_eq!(String::from("svg"), get_extension_from_filename("", "svg"));

        assert_eq!(String::from("atxt"), get_extension_from_filename("", "txt"));

        // Plantuml does this 'braille.png' extension
        assert_eq!(
            String::from("braille.png"),
            get_extension_from_filename("", "braille")
        );

        {
            // ditaa graphs
            // Note the format is overridden when rendering ditaa
            assert_eq!(
                String::from("png"),
                get_extension_from_filename("@startditaa", "svg")
            );

            assert_eq!(
                String::from("png"),
                get_extension_from_filename("@startditaa", "png")
            );

            assert_eq!(
                String::from("png"),
                get_extension_from_filename(
                    "Also when not at the start of the code block @startditaa",
                    "svg"
                )
            );
        }
    }

    #[test]
    fn test_get_image_filename() {
        let code = String::from("asgtfgl");
        let file_path = get_image_filename(Path::new("foo"), &code, "svg");
        assert_eq!(PathBuf::from("foo"), file_path.parent().unwrap());
        assert_eq!(
            sha1::Sha1::from(&code).hexdigest(),
            file_path.file_stem().unwrap().to_str().unwrap()
        );
        assert_eq!(PathBuf::from("svg"), file_path.extension().unwrap());
    }
}
