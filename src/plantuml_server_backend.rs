use base64_plantuml::Base64PlantUML;
use deflate::deflate_bytes;
use failure::Error;
use plantuml_backend::{get_extension, get_image_filename, PlantUMLBackend};
use reqwest;
use reqwest::Url;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

/// Helper trait for unit testing purposes (allow testing without a live server)
trait ImageDownloader {
    fn download_image(&self, request_url: &Url) -> Result<Vec<u8>, Error>;
}

struct RealImageDownloader;

impl ImageDownloader for RealImageDownloader {
    /// Download the image at the given URL, return the response body as a
    /// Vec<u8>
    fn download_image(&self, request_url: &Url) -> Result<Vec<u8>, Error> {
        let mut image_buf: Vec<u8> = vec![];
        reqwest::get(request_url.clone())
            .and_then(|mut response| response.copy_to(&mut image_buf))
            .and_then(|_| Ok(image_buf))
            .or_else(|e| bail!(format!("Failed to generate diagram ({})", e)))
    }
}

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

    /// Format the PlantUML server URL using the encoded diagram and extension
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

    /// Save the downlaoded image to a file and return the relative image URL
    /// for use in the mdBook
    fn save_downloaded_image(
        &self,
        image_buffer: &Vec<u8>,
        extension: &String,
    ) -> Result<PathBuf, Error> {
        let filename = get_image_filename(&self.img_root, extension);
        let mut output_file = fs::File::create(&filename)?;
        output_file.write_all(&image_buffer)?;

        Ok(filename)
    }

    /// The business end of this struct, generate the image using the server and
    /// return the relative image URL.
    fn render_string(
        &self,
        plantuml_code: &String,
        downloader: &dyn ImageDownloader,
    ) -> Result<PathBuf, Error> {
        let encoded = encode_diagram_source(plantuml_code);
        let extension = get_extension(plantuml_code);
        let request_url = self.get_url(&extension, &encoded)?;

        match downloader.download_image(&request_url) {
            Ok(image_buffer) => self.save_downloaded_image(&image_buffer, &extension),
            Err(e) => Err(e),
        }
    }
}

/// Compress and encode the image source, return the encoed Base64-ish string
fn encode_diagram_source(plantuml_code: &String) -> String {
    let compressed = deflate_bytes(&plantuml_code.as_bytes());
    let base64_compressed = Base64PlantUML::encode(&compressed);

    base64_compressed
}

impl PlantUMLBackend for PlantUMLServer {
    fn render_from_string(&self, plantuml_code: &String) -> Result<PathBuf, Error> {
        let downloader = RealImageDownloader {};
        self.render_string(plantuml_code, &downloader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simulacrum::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_url() {
        let srv = PlantUMLServer::new(
            Url::parse("http://froboz:1234/plantuml").unwrap(),
            PathBuf::from(""),
        );

        assert_eq!(
            Url::parse("http://froboz:1234/plantuml/ext/plantuml_encoded_string").unwrap(),
            srv.get_url(
                &String::from("ext"),
                &String::from("plantuml_encoded_string")
            )
            .unwrap()
        );

        // I cannot manage Url::parse to fail using the ext and encoded data
        // parts :-(. It automatically encodes the invalid characters in the url
        // when parsing. So no test for the error case.
    }

    #[test]
    fn test_encode_diagram_source() {
        assert_eq!(
            String::from("SrRGrQsnKt010000"),
            encode_diagram_source(&String::from("C --|> D"))
        )
    }

    #[test]
    fn test_save_downloaded_image() {
        let tmp_dir = tempdir().unwrap();
        let output_path = tmp_dir.into_path();
        let srv = PlantUMLServer::new(Url::parse("http://froboz").unwrap(), output_path.clone());

        let data: Vec<u8> = b"totemizer".iter().cloned().collect();
        let img_path = srv
            .save_downloaded_image(&data, &String::from("ext"))
            .unwrap();

        let raw_source = fs::read(img_path).unwrap();
        assert_eq!("totemizer", String::from_utf8_lossy(&raw_source));
    }

    create_mock! {
        impl ImageDownloader for ImageDownloaderMock (self) {
            expect_download_image("download_image"):
                fn download_image(&self, request_url: &Url) -> Result<Vec<u8>, Error>;
        }
    }

    #[test]
    fn test_render_string() {
        let tmp_dir = tempdir().unwrap();
        let output_path = tmp_dir.into_path();
        let srv = PlantUMLServer::new(Url::parse("http://froboz").unwrap(), output_path.clone());

        let mut mock_downloader = ImageDownloaderMock::new();
        mock_downloader
            .expect_download_image()
            .called_once()
            //.with(...) How to test the correct Url here?
            .returning(|_| Ok(b"the rendered image".iter().cloned().collect()));

        let img_path = srv
            .render_string(&String::from("C --|> D"), &mock_downloader)
            .unwrap();

        let raw_source = fs::read(img_path).unwrap();
        assert_eq!("the rendered image", String::from_utf8_lossy(&raw_source));
    }
}
