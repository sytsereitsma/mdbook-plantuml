use std::fs;
use std::path::PathBuf;

use failure::Error;
use plantuml_server_backend::PlantUMLServer;
use plantuml_shell_backend::PlantUMLShell;
use plantumlconfig::PlantUMLConfig;
use reqwest::Url;
use uuid::Uuid;

pub trait PlantUMLBackend {
    ///Render a PlantUML string to file and return the diagram URL path to this
    ///file (as a String) for use in a link.
    fn render_from_string(&self, s: &String, rel_img_url: &String) -> Result<String, Error>;
}

///Get the preferred extension. Default is svg to allow maximum resolution on
///all zoom levels. Some diagrams, like ditaa, cannot be rendered in svg by
///PlantUML, so we return 'png' for these.
pub fn get_extension(plantuml_code: &String) -> String {
    if plantuml_code.contains("@startditaa") {
        String::from("png")
    } else {
        String::from("svg")
    }
}

/// Create the image names with the appropriate extension and path
/// The base name of the file is a UUID to avoid collisions with existing
/// files
pub fn get_image_filename(img_root: &PathBuf, extension: &String) -> PathBuf {
    let mut output_file = img_root.clone();
    output_file.push(Uuid::new_v4().to_string());
    output_file.set_extension(extension);

    output_file
}

/// Create an instance of the PlantUMLBackend
/// For now only a PlantUMLShell instance is created, later server support will be added
pub fn create(cfg: &PlantUMLConfig, book_root: &PathBuf) -> Box<PlantUMLBackend> {
    let cmd = match &cfg.plantuml_cmd {
        Some(s) => s.clone(),
        None => {
            if cfg!(target_os = "windows") {
                String::from("java -jar plantuml.jar")
            } else {
                String::from("/usr/bin/plantuml")
            }
        }
    };

    //Always create the image output dir
    let mut img_root = book_root.clone();
    img_root.push("img");
    fs::create_dir_all(&img_root).expect("Failed to create image output dir.");

    if let Ok(server_url) = Url::parse(&cmd) {
        Box::new(PlantUMLServer::new(server_url, img_root))
    } else {
        Box::new(PlantUMLShell::new(cmd, img_root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_extension() {
        assert_eq!(
            String::from("svg"),
            get_extension(&String::from("C --|> D"))
        );

        assert_eq!(
            String::from("png"),
            get_extension(&String::from("@startditaa"))
        );

        assert_eq!(
            String::from("png"),
            get_extension(&String::from(
                "Also when not at the start of the code block @startditaa"
            ))
        );
    }

    #[test]
    fn test_get_image_filename() {
        let file_path = get_image_filename(&PathBuf::from("foo"), &String::from("bar"));

        assert_eq!(PathBuf::from("foo"), file_path.parent().unwrap());
        assert_eq!(PathBuf::from("bar"), file_path.extension().unwrap());
        assert_eq!(36, file_path.file_stem().unwrap().len());
    }
}
