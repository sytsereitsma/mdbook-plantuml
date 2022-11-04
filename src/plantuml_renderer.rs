use crate::dir_cleaner::DirCleaner;
use crate::plantuml_backend::PlantUMLBackend;
use crate::plantuml_backend_factory;
use crate::plantumlconfig::PlantUMLConfig;
use anyhow::{Context, Result};
use base64::encode;
use sha1::{Digest, Sha1};
use std::cell::RefCell;
use std::fs;

use std::path::{Path, PathBuf};

pub trait PlantUMLRendererTrait {
    fn render(
        &self,
        plantuml_code: &str,
        rel_img_url: &str,
        image_format: String,
    ) -> Result<String>;
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
    let mut output_file = img_root.join(hash_string(plantuml_code));
    output_file.set_extension(extension);

    output_file
}

fn hash_string(code: &str) -> String {
    let hash = Sha1::new_with_prefix(code).finalize();
    base16ct::lower::encode_string(&hash)
}

pub struct PlantUMLRenderer {
    backend: Box<dyn PlantUMLBackend>,
    cleaner: RefCell<DirCleaner>,
    img_root: PathBuf,
    clickable_img: bool,
    use_data_uris: bool,
}

impl PlantUMLRenderer {
    pub fn new(cfg: &PlantUMLConfig, img_root: PathBuf) -> Self {
        let renderer = Self {
            backend: plantuml_backend_factory::create(cfg),
            cleaner: RefCell::new(DirCleaner::new(img_root.as_path())),
            img_root: img_root,
            clickable_img: cfg.clickable_img,
            use_data_uris: cfg.use_data_uris,
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

    fn create_datauri(image_path: &PathBuf) -> Result<String> {
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs#syntax

        let media_type = match image_path
            .extension()
            .map(|s| s.to_str())
            .unwrap_or(Some(""))
        {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("svg") => "image/svg+xml",
            Some("atxt") | Some("utxt") | Some("txt") => "text/plain",
            _ => "",
        };

        let image_data = fs::read(image_path)
            .with_context(|| format!("Could not open image file {:?}", image_path))?;
        let encoded_value = encode(&image_data);
        Ok(format!("data:{};base64,{}", media_type, encoded_value))
    }

    fn create_image_datauri_element(image_path: &PathBuf, clickable: bool) -> Result<String> {
        let uri = PlantUMLRenderer::create_datauri(image_path)?;
        if clickable {
            // Note that both Edge and Firefox do not allow clicking on data URI links
            // So this probably won't work. Kept in here regardless for consistency
            Ok(format!("[![]({})]({})\n\n", uri, uri))
        } else {
            Ok(format!("![]({})\n\n", uri))
        }
    }

    fn create_inline_txt_image(image_path: &Path) -> Result<String> {
        log::debug!("Creating inline image from {:?}", image_path);
        let raw_source = fs::read(image_path).unwrap();
        let txt = String::from_utf8(raw_source)?;

        Ok(format!("\n```txt\n{}```\n", txt))
    }

    pub fn render(
        &self,
        plantuml_code: &str,
        rel_img_url: &str,
        image_format: &str,
    ) -> Result<String> {
        // When operating in data-uri mode the images are written to in .mdbook-plantuml, otherwise
        // they are written to src/mdbook-plantuml-images (cannot write to the book output dir, because
        // mdbook deletes the files in there after preprocessing)
        let output_file = get_image_filename(&self.img_root, plantuml_code, image_format);
        if !output_file.exists() {
            // File is not cached, render the image
            let data = self
                .backend
                .render_from_string(plantuml_code, image_format)?;

            // Save the file even if we inline images
            std::fs::write(&output_file, &data).with_context(|| {
                format!(
                    "Failed to save PlantUML diagram to {}.",
                    output_file.to_string_lossy()
                )
            })?;
        }

        // Let the dir cleaner know this file should be kept
        self.cleaner.borrow_mut().keep(&output_file);

        let extension = output_file.extension().unwrap_or_default();
        if extension == "atxt" || extension == "utxt" {
            Self::create_inline_txt_image(&output_file)
        } else if self.use_data_uris {
            Self::create_image_datauri_element(&output_file, self.clickable_img)
        } else {
            Ok(Self::create_md_link(
                rel_img_url,
                &output_file,
                self.clickable_img,
            ))
        }
    }
}

impl PlantUMLRendererTrait for PlantUMLRenderer {
    fn render(
        &self,
        plantuml_code: &str,
        rel_img_url: &str,
        image_format: String,
    ) -> Result<String> {
        Self::render(self, plantuml_code, rel_img_url, &image_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{bail, Result};
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::Write;
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

    #[test]
    fn test_create_datauri() {
        let temp_directory = tempdir().unwrap();
        let content = "test content";

        let svg_path = temp_directory.path().join("file.svg");
        let mut svg_file = File::create(&svg_path).unwrap();
        writeln!(svg_file, "{}", content).unwrap();
        drop(svg_file); // Close and flush content to file
        assert_eq!(
            String::from("data:image/svg+xml;base64,dGVzdCBjb250ZW50Cg=="),
            PlantUMLRenderer::create_datauri(&svg_path)
        );

        let png_path = temp_directory.path().join("file.png");
        let mut png_file = File::create(&png_path).unwrap();
        writeln!(png_file, "{}", content).unwrap();
        drop(png_file); // Close and flush content to file
        assert_eq!(
            String::from("data:image/png;base64,dGVzdCBjb250ZW50Cg=="),
            PlantUMLRenderer::create_datauri(&png_path)
        );

        let txt_path = temp_directory.path().join("file.txt");
        let mut txt_file = File::create(&txt_path).unwrap();
        writeln!(txt_file, "{}", content).unwrap();
        drop(txt_file); // Close and flush content to file
        assert_eq!(
            String::from("data:text/plain;base64,dGVzdCBjb250ZW50Cg=="),
            PlantUMLRenderer::create_datauri(&txt_path)
        );

        let jpeg_path = temp_directory.path().join("file.jpeg");
        let mut jpeg_file = File::create(&jpeg_path).unwrap();
        writeln!(jpeg_file, "{}", content).unwrap();
        drop(jpeg_file); // Close and flush content to file
        assert_eq!(
            String::from("data:image/jpeg;base64,dGVzdCBjb250ZW50Cg=="),
            PlantUMLRenderer::create_datauri(&jpeg_path)
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
        ) -> Result<Vec<u8>> {
            if self.is_ok {
                std::fs::write(output_file, format!("{}\n{}", plantuml_code, image_format))?;

                return Ok(fs::read(&output_file)?);
            }
            bail!("Oh no")
        }
    }

    #[test]
    fn test_rendering_md_link() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(DirCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
            use_data_uris: false,
        };

        let plantuml_code = "some puml code";
        let code_hash = hash_string(plantuml_code);

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
    fn test_rendering_datauri() {
        let output_dir = tempdir().unwrap();
        let renderer = PlantUMLRenderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(DirCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
            use_data_uris: true,
        };

        let plantuml_code = "some puml code";

        // svg extension
        assert_eq!(
            format!(
                "![]({})\n\n",
                "data:image/svg+xml;base64,c29tZSBwdW1sIGNvZGUKc3Zn"
            ),
            renderer.render(&plantuml_code, "rel/url", "svg")
        );

        // png extension
        assert_eq!(
            format!(
                "![]({})\n\n",
                "data:image/png;base64,c29tZSBwdW1sIGNvZGUKcG5n"
            ),
            renderer.render(&plantuml_code, "rel/url", "png")
        );

        // txt extension
        assert_eq!(
            String::from("\n```txt\nsome puml code\ntxt```\n"),
            renderer.render(&plantuml_code, "rel/url", "txt")
        );

        // utxt extension
        assert_eq!(
            String::from("\n```txt\nsome puml code\ntxt```\n"),
            renderer.render(&plantuml_code, "rel/url", "txt")
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
            use_data_uris: false,
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
        let code = "asgtfgl";
        let file_path = get_image_filename(Path::new("foo"), code, "svg");
        assert_eq!(PathBuf::from("foo"), file_path.parent().unwrap());
        assert_eq!(
            hash_string(code),
            file_path.file_stem().unwrap().to_str().unwrap()
        );
        assert_eq!(PathBuf::from("svg"), file_path.extension().unwrap());
    }
}
