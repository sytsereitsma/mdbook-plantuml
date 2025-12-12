use crate::backend::{self, Backend};
use crate::cache_cleaner::CacheCleaner;
use crate::config::Config;
use crate::include_iterator::IncludeIterator;
use anyhow::{Context, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use sha1::{Digest, Sha1};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

pub trait RendererTrait {
    fn render(
        &self,
        plantuml_code: &str,
        rel_img_url: &str,
        image_format: String,
        // Try to inline the image (only works for SVG images)
        inline: bool,
    ) -> Result<String>;
}

/// Create the image names with the appropriate extension and path
/// The base name of the file is a SHA1 of the code block to avoid collisions
/// with existing and as a bonus prevent duplicate files.
pub fn image_filename(img_root: &Path, plantuml_code: &str, image_format: &str) -> PathBuf {
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
    let mut output_file = img_root.join(create_hash_from_code(plantuml_code));
    output_file.set_extension(extension);

    output_file
}

/// Create a SHA1 hash from the given PlantUML code
/// When the code contains !include or !includesub rules try to load the include files and add
/// them to the hash so that changes in those files are picked up by the caching system.
fn create_hash_from_code(code: &str) -> String {
    let mut hash = Sha1::new_with_prefix(code);
    let include_iter = IncludeIterator::new(code);

    for include_file in include_iter {
        if let Ok(include_data) = fs::read_to_string(include_file) {
            hash.update(include_data);
        } else {
            // Do not fail the rendering when an include file cannot be read
            // Sprites do not represent files on the filesystem
            log::warn!(
                "Could not read included file '{}' for caching (if the include is a sprite or non-filesystem url this is expected behavior)",
                include_file
            );
        }
    }

    base16ct::lower::encode_string(&hash.finalize())
}

pub struct Renderer {
    backend: Box<dyn Backend>,
    cleaner: RefCell<CacheCleaner>,
    img_root: PathBuf,
    clickable_img: bool,
    use_data_uris: bool,
}

impl Renderer {
    pub fn new(cfg: &Config, img_root: PathBuf) -> Self {
        Self {
            backend: backend::factory::create(cfg),
            cleaner: RefCell::new(CacheCleaner::new(img_root.as_path())),
            img_root,
            clickable_img: cfg.clickable_img,
            use_data_uris: cfg.use_data_uris,
        }
    }

    fn create_md_link(rel_img_url: &str, image_path: &Path, clickable: bool) -> String {
        let img_url = format!(
            "{}/{}",
            rel_img_url,
            image_path.file_name().unwrap().to_str().unwrap()
        );
        if clickable {
            format!("[![]({img_url})]({img_url})\n\n")
        } else {
            format!("![]({img_url})\n\n")
        }
    }

    fn create_datauri(image_path: &Path) -> Result<String> {
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs#syntax

        let media_type = match image_path
            .extension()
            .map(|s| s.to_str())
            .unwrap_or(Some(""))
        {
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("svg") => "image/svg+xml",
            Some("atxt" | "utxt" | "txt") => "text/plain",
            _ => "",
        };

        let image_data = fs::read(image_path)
            .with_context(|| format!("Could not open image file {image_path:?}"))?;
        let encoded_value = BASE64_STANDARD.encode(image_data);
        Ok(format!("data:{media_type};base64,{encoded_value}"))
    }

    fn create_image_datauri_element(image_path: &Path, clickable: bool) -> Result<String> {
        let uri = Self::create_datauri(image_path)?;
        if clickable {
            // Note that both Edge and Firefox do not allow clicking on data URI links
            // So this probably won't work. Kept in here regardless for consistency
            Ok(format!("[![]({uri})]({uri})\n\n"))
        } else {
            Ok(format!("![]({uri})\n\n"))
        }
    }

    fn create_inline_txt_image(image_path: &Path) -> Result<String> {
        log::debug!("Creating inline image from {:?}", image_path);
        let raw_source = fs::read(image_path).unwrap();
        let txt = String::from_utf8(raw_source)?;

        Ok(format!("\n```txt\n{txt}```\n"))
    }

    fn create_inline_svg_image(image_path: &Path) -> Result<String> {
        log::debug!("Creating inline svg image from {:?}", image_path);
        let raw_source = fs::read(image_path).unwrap();
        let svg = String::from_utf8(raw_source)?;

        Ok(svg)
    }

    pub fn render(
        &self,
        plantuml_code: &str,
        rel_img_url: &str,
        image_format: &str,
        inline: bool,
    ) -> Result<String> {
        // When operating in data-uri mode the images are written to in .mdbook-plantuml, otherwise
        // they are written to src/mdbook-plantuml-images (cannot write to the book output dir, because
        // mdbook deletes the files in there after preprocessing)
        let output_file = image_filename(&self.img_root, plantuml_code, image_format);
        if !output_file.exists() {
            log::debug!("Regenerating image file {:?}", output_file);
            // File is not cached, render the image
            let data = self
                .backend
                .render_from_string(plantuml_code, image_format)?;

            // Save the file even if we inline images
            std::fs::write(&output_file, data).with_context(|| {
                format!(
                    "Failed to save PlantUML diagram to {}.",
                    output_file.to_string_lossy()
                )
            })?;
        } else {
            log::debug!("Using cached image file {:?}", output_file);
        }

        // Let the dir cleaner know this file should be kept
        self.cleaner.borrow_mut().keep(&output_file);

        let extension = output_file.extension().unwrap_or_default();
        if extension == "atxt" || extension == "utxt" {
            Self::create_inline_txt_image(&output_file)
        } else if extension == "svg" && inline {
            // Inlining SVG images allows the use of links embedded in the SVG
            Self::create_inline_svg_image(&output_file)
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

impl RendererTrait for Renderer {
    fn render(
        &self,
        plantuml_code: &str,
        rel_img_url: &str,
        image_format: String,
        inline: bool,
    ) -> Result<String> {
        Self::render(self, plantuml_code, rel_img_url, &image_format, inline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, bail};
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_create_md_link() {
        assert_eq!(
            String::from("![](foo/bar/baz.svg)\n\n"),
            Renderer::create_md_link("foo/bar", Path::new("/froboz/baz.svg"), false)
        );

        assert_eq!(
            "![](/baz.svg)\n\n",
            Renderer::create_md_link("", Path::new("baz.svg"), false)
        );

        assert_eq!(
            String::from("![](/baz.svg)\n\n"),
            Renderer::create_md_link("", Path::new("foo/baz.svg"), false)
        );
    }

    #[test]
    fn test_create_datauri() {
        let temp_directory = tempdir().unwrap();
        let content = "test content";

        let svg_path = temp_directory.path().join("file.svg");
        let mut svg_file = File::create(&svg_path).unwrap();
        writeln!(svg_file, "{content}").unwrap();
        drop(svg_file); // Close and flush content to file
        assert_eq!(
            String::from("data:image/svg+xml;base64,dGVzdCBjb250ZW50Cg=="),
            Renderer::create_datauri(&svg_path).unwrap()
        );

        let png_path = temp_directory.path().join("file.png");
        let mut png_file = File::create(&png_path).unwrap();
        writeln!(png_file, "{content}").unwrap();
        drop(png_file); // Close and flush content to file
        assert_eq!(
            String::from("data:image/png;base64,dGVzdCBjb250ZW50Cg=="),
            Renderer::create_datauri(&png_path).unwrap()
        );

        let txt_path = temp_directory.path().join("file.txt");
        let mut txt_file = File::create(&txt_path).unwrap();
        writeln!(txt_file, "{content}").unwrap();
        drop(txt_file); // Close and flush content to file
        assert_eq!(
            String::from("data:text/plain;base64,dGVzdCBjb250ZW50Cg=="),
            Renderer::create_datauri(&txt_path).unwrap()
        );

        let jpeg_path = temp_directory.path().join("file.jpeg");
        let mut jpeg_file = File::create(&jpeg_path).unwrap();
        writeln!(jpeg_file, "{content}").unwrap();
        drop(jpeg_file); // Close and flush content to file
        assert_eq!(
            String::from("data:image/jpeg;base64,dGVzdCBjb250ZW50Cg=="),
            Renderer::create_datauri(&jpeg_path).unwrap()
        );
    }

    struct BackendMock {
        is_ok: bool,
    }

    impl Backend for BackendMock {
        fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>> {
            if self.is_ok {
                return Ok(Vec::from(
                    format!("{plantuml_code}\n{image_format}").as_bytes(),
                ));
            }
            bail!("Oh no");
        }
    }

    #[test]
    fn test_rendering_md_link() {
        let output_dir = tempdir().unwrap();
        let renderer = Renderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(CacheCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
            use_data_uris: false,
        };

        let plantuml_code = "some puml code";
        let code_hash = create_hash_from_code(plantuml_code);

        assert_eq!(
            format!("![](rel/url/{code_hash}.svg)\n\n"),
            renderer
                .render(plantuml_code, "rel/url", "svg", false)
                .unwrap()
        );

        // png extension
        assert_eq!(
            format!("![](rel/url/{code_hash}.png)\n\n"),
            renderer
                .render(plantuml_code, "rel/url", "png", false)
                .unwrap()
        );

        // txt extension
        assert_eq!(
            format!("\n```txt\n{plantuml_code}\ntxt```\n"), /* image format is appended by
                                                             * fake backend */
            renderer
                .render(plantuml_code, "rel/url", "txt", false)
                .unwrap()
        );

        // utxt extension
        assert_eq!(
            format!("\n```txt\n{plantuml_code}\ntxt```\n"), /* image format is appended by
                                                             * fake backend */
            renderer
                .render(plantuml_code, "rel/url", "txt", false)
                .unwrap()
        );
    }
    #[test]
    fn test_rendering_inline_svg() {
        let output_dir = tempdir().unwrap();
        let renderer = Renderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(CacheCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
            use_data_uris: false,
        };

        let plantuml_code = "some puml code";

        // With inlining
        assert_eq!(
            format!("{}{}", plantuml_code, "\nsvg"),
            renderer
                .render(plantuml_code, "rel/url", "svg", true)
                .unwrap()
        );

        // Without inlining
        let code_hash = create_hash_from_code(plantuml_code);
        assert_eq!(
            format!("![](rel/url/{code_hash}.svg)\n\n"),
            renderer
                .render(plantuml_code, "rel/url", "svg", false)
                .unwrap()
        );
    }

    #[test]
    fn test_rendering_datauri() {
        let output_dir = tempdir().unwrap();
        let renderer = Renderer {
            backend: Box::new(BackendMock { is_ok: true }),
            cleaner: RefCell::new(CacheCleaner::new(output_dir.path())),
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
            renderer
                .render(plantuml_code, "rel/url", "svg", false)
                .unwrap()
        );

        // png extension
        assert_eq!(
            format!(
                "![]({})\n\n",
                "data:image/png;base64,c29tZSBwdW1sIGNvZGUKcG5n"
            ),
            renderer
                .render(plantuml_code, "rel/url", "png", false)
                .unwrap()
        );

        // txt extension
        assert_eq!(
            String::from("\n```txt\nsome puml code\ntxt```\n"),
            renderer
                .render(plantuml_code, "rel/url", "txt", false)
                .unwrap()
        );

        // utxt extension
        assert_eq!(
            String::from("\n```txt\nsome puml code\ntxt```\n"),
            renderer
                .render(plantuml_code, "rel/url", "txt", false)
                .unwrap()
        );
    }

    #[test]
    fn test_rendering_failure() {
        let output_dir = tempdir().unwrap();
        let renderer = Renderer {
            backend: Box::new(BackendMock { is_ok: false }),
            cleaner: RefCell::new(CacheCleaner::new(output_dir.path())),
            img_root: output_dir.path().to_path_buf(),
            clickable_img: false,
            use_data_uris: false,
        };

        let result = renderer.render("", "rel/url", "svg", false);
        let error_str = format!("{}", result.err().unwrap());
        assert_eq!("Oh no", error_str);
    }

    #[test]
    fn test_image_filename_extension() {
        let extension_from_filename = |code: &str, img_format: &str| -> String {
            let file_path = image_filename(Path::new("foo"), code, img_format)
                .to_string_lossy()
                .to_string();
            let firstdot = file_path.find('.').unwrap();
            file_path[firstdot + 1..].to_string()
        };

        assert_eq!(String::from("svg"), extension_from_filename("", "svg"));

        assert_eq!(String::from("eps"), extension_from_filename("", "eps"));

        assert_eq!(String::from("png"), extension_from_filename("", "png"));

        assert_eq!(String::from("svg"), extension_from_filename("", ""));

        assert_eq!(String::from("svg"), extension_from_filename("", "svg"));

        assert_eq!(String::from("atxt"), extension_from_filename("", "txt"));

        // Plantuml does this 'braille.png' extension
        assert_eq!(
            String::from("braille.png"),
            extension_from_filename("", "braille")
        );

        {
            // ditaa graphs
            // Note the format is overridden when rendering ditaa
            assert_eq!(
                String::from("png"),
                extension_from_filename("@startditaa", "svg")
            );

            assert_eq!(
                String::from("png"),
                extension_from_filename("@startditaa", "png")
            );

            assert_eq!(
                String::from("png"),
                extension_from_filename(
                    "Also when not at the start of the code block @startditaa",
                    "svg"
                )
            );
        }
    }

    #[test]
    fn test_image_filename() {
        let code = "asgtfgl";
        let file_path = image_filename(Path::new("foo"), code, "svg");
        assert_eq!(PathBuf::from("foo"), file_path.parent().unwrap());
        assert_eq!(
            create_hash_from_code(code),
            file_path.file_stem().unwrap().to_str().unwrap()
        );
        assert_eq!(PathBuf::from("svg"), file_path.extension().unwrap());
    }

    #[test]
    fn test_create_hash_from_code_no_include() {
        let code = "@startuml\nAlice -> Bob: Hello\n@enduml";
        let hash = create_hash_from_code(code);
        assert_eq!("79b57dbdefc431bfab3f4f17c032d39823cbd210", hash);

        // Different code, different hash
        let code = "@startuml\nBob -> Alice: Hello\n@enduml";
        let hash = create_hash_from_code(code);
        assert_eq!("059720b7027e8d7af44cdbabee7d47ae1277cd83", hash);
    }

    #[test]
    fn test_create_hash_from_code_includes() {
        // This test is a bit tricky. The tempdir is different every time, which means the include/includesub
        // paths are different every time, which in turn means the hash will be different every time.
        // So first do a run with some include files, check the hash, then change the include files
        // and check the hash changes.
        let include_dir = tempdir().unwrap();
        let include_file_path = include_dir.path().join("include.puml");
        let include_sub_file_path = include_dir.path().join("include_sub.puml");

        let write_file = |path: &Path, content: &str| -> Result<()> {
            let mut file = File::create(path)?;
            file.write_all(content.as_bytes())?;
            Ok(())
        };

        let create_baseline = || -> String {
            write_file(&include_file_path, "goats").unwrap();
            write_file(&include_sub_file_path, "easels").unwrap();

            format!(
                "@startuml\n  !include {}\n!includesub {}!FOO\nAlice -> Bob: Hello\n@enduml",
                include_file_path.display(),
                include_sub_file_path.display()
            )
        };

        let code = create_baseline();
        let baseline_hash = create_hash_from_code(&code);

        // Now change the include file, the hash should change
        write_file(&include_file_path, "goats with pants").unwrap();
        let include_hash = create_hash_from_code(&code);
        assert_ne!(baseline_hash, include_hash);

        // Now change the includesub file, the hash should change
        write_file(&include_sub_file_path, "easels with hats").unwrap();
        let includesub_hash = create_hash_from_code(&code);
        assert_ne!(include_hash, includesub_hash);

        // Finally change the code block itself, the hash should change
        let code = code.to_owned() + "\n' A comment";
        let hash = create_hash_from_code(&code);
        assert_ne!(includesub_hash, hash);

        // As a final check reset to the baseline code and files, the hash should be the same as the original
        let code = create_baseline();
        let final_hash = create_hash_from_code(&code);
        assert_eq!(baseline_hash, final_hash);
    }

    #[test]
    fn test_create_hash_from_code_includes_when_includes_cannot_be_found() {
        // Two include files that do not exist are referenced
        let code = "@startuml\n!include not-here.puml\n!includesub not-here-either.puml!FOO\nAlice -> Bob: Hello\n@enduml";
        let hash = create_hash_from_code(code);
        assert_eq!("9183290693ec58cf6897b718a376e5b898a17f88", hash);

        // Change the code block itself, the hash should change
        let code = code.to_owned() + "\n' A comment";
        let hash = create_hash_from_code(code.as_str());
        assert_eq!("07bdd9d54b4662ef657b0b94f147641d4f9c464b", hash);
    }
}
