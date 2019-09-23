use deflate::deflate_bytes;
use failure::Error;
use plantuml_backend::{get_extension, PlantUMLBackend};
use reqwest;
use reqwest::Url;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use uuid::Uuid;

use base64_plantuml::Base64PlantUML;

pub struct PlantUMLServer {
    server_url: Url,
    img_root: PathBuf,
}

impl PlantUMLServer {
    pub fn new(server_url: Url, img_root: PathBuf) -> PlantUMLServer {
        PlantUMLServer {
            server_url: server_url,
            img_root: img_root,
        }
    }

    /// Create the source and image names with the appropriate extensions
    /// The file base names are a UUID to avoid collisions with exsisting
    /// files
    fn get_image_filename(&self, extension: &String) -> PathBuf {
        let mut output_file = self.img_root.clone();
        output_file.push(Uuid::new_v4().to_string());
        output_file.set_extension(extension);

        output_file
    }

    fn get_url(&self, extension: &String, encoded_diagram: &String) -> Result<Url, Error> {
        let formatted_url = format!(
            "{}/{}/{}",
            self.server_url.as_str(),
            extension,
            encoded_diagram
        );
        match Url::parse(formatted_url.as_str()) {
            Ok(url) => Ok(url),
            Err(e) => bail!(format!(
                "Error parsing PlantUML server URL from '{}' ({})",
                formatted_url, e
            )),
        }
    }

    fn save_image(&self, image_buffer: &Vec<u8>, extension: &String) -> Result<String, Error> {
        let filename = self.get_image_filename(extension);
        let mut output_file = File::create(&filename)?;
        output_file.write_all(&image_buffer)?;

        Ok(format!(
            "img/{}",
            filename.file_name().unwrap().to_str().unwrap()
        ))
    }

    fn render_string(&self, plantuml_code: &String) -> Result<String, Error> {
        let encoded = encode_diagram_source(plantuml_code);
        let extension = get_extension(plantuml_code);
        let request_url = self.get_url(&extension, &encoded)?;
        fs::write("foo.txt", request_url.as_str())?;
        match download_image(&request_url) {
            Ok(image_buffer) => self.save_image(&image_buffer, &extension),
            Err(e) => Err(e),
        }
    }
}

fn download_image(request_url: &Url) -> Result<Vec<u8>, Error> {
    let mut image_buf: Vec<u8> = vec![];
    reqwest::get(request_url.clone())
        .and_then(|mut response| response.copy_to(&mut image_buf))
        .and_then(|_| Ok(image_buf))
        .or_else(|e| bail!(format!("Failed to generate diagram ({})", e)))
}

fn encode_diagram_source(plantuml_code: &String) -> String {
    let compressed = deflate_bytes(&plantuml_code.as_bytes());
    let base64_compressed = Base64PlantUML::encode(&compressed);

    base64_compressed
}

//http://localhost:8080/plantuml/svg/SyfFKj2rKt3CoKnELR1Io4ZDoSa70000

impl PlantUMLBackend for PlantUMLServer {
    fn render_from_string(&self, plantuml_code: &String) -> Result<String, Error> {
        self.render_string(plantuml_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn encodes_diagram_source() {
        assert_eq!(
            String::from("SrRGrQsnKt010000"),
            encode_diagram_source(&String::from("C --|> D"))
        )
    }

}
