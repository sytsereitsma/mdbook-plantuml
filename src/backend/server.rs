use crate::backend::Backend;
use crate::base64;
use anyhow::{bail, Result};
use deflate::deflate_bytes;
use reqwest::Url;

/// Helper trait for unit testing purposes (allow testing without a live server)
trait ImageDownloader {
    fn download_image(&self, request_url: &Url) -> Result<Vec<u8>>;
}

struct RealImageDownloader;

impl ImageDownloader for RealImageDownloader {
    /// Download the image at the given URL, return the response body as a
    /// Vec<u8>
    fn download_image(&self, request_url: &Url) -> Result<Vec<u8>> {
        let mut image_buf: Vec<u8> = vec![];
        reqwest::blocking::get(request_url.clone())
            .and_then(|mut response| response.copy_to(&mut image_buf))
            .or_else(|e| bail!("Failed to generate diagram ({})", e))?;
        Ok(image_buf)
    }
}

pub struct PlantUMLServer {
    server_url: Url,
}

impl PlantUMLServer {
    pub fn new(server_url: Url) -> Self {
        // Make sure the server_url path ends with a / so Url::join works as expected
        // later.
        let path = server_url.path();
        let server_url = if path.ends_with('/') {
            server_url
        } else {
            let mut repath = server_url.clone();
            repath.set_path(format!("{path}/").as_str());
            repath
        };

        Self { server_url }
    }

    /// Format the PlantUML server URL using the encoded diagram and extension
    fn url(&self, image_format: &str, encoded_diagram: &str) -> Result<Url> {
        let path = format!("{image_format}/{encoded_diagram}");

        self.server_url.join(&path).map_err(|e| {
            anyhow::format_err!(
                "Error constructing PlantUML server URL from '{}' and '{}' ({})",
                self.server_url.as_str(),
                path,
                e
            )
        })
    }

    /// The business end of this struct, generate the image using the server and
    /// return the relative image URL.
    fn render_string(
        &self,
        plantuml_code: &str,
        image_format: &str,
        downloader: &dyn ImageDownloader,
    ) -> Result<Vec<u8>> {
        let encoded = encode_diagram_source(plantuml_code);
        let request_url = self.url(image_format, &encoded)?;

        downloader.download_image(&request_url)
    }
}

/// Compress and encode the image source, return the encoed Base64-ish string
fn encode_diagram_source(plantuml_code: &str) -> String {
    let compressed = deflate_bytes(plantuml_code.as_bytes());
    base64::encode(&compressed)
}

impl Backend for PlantUMLServer {
    fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>> {
        let downloader = RealImageDownloader {};
        self.render_string(plantuml_code, image_format, &downloader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use simulacrum::*;

    #[test]
    fn test_url() {
        let srv = PlantUMLServer::new(Url::parse("http://froboz:1234/plantuml").unwrap());

        assert_eq!(
            Url::parse("http://froboz:1234/plantuml/ext/plantuml_encoded_string").unwrap(),
            srv.url("ext", "plantuml_encoded_string").unwrap()
        );

        // I cannot manage Url::parse to fail using the ext and encoded data
        // parts :-(. It automatically encodes the invalid characters in the url
        // when parsing. So no test for the error case.
    }

    #[test]
    fn test_url_no_path() {
        let srv = PlantUMLServer::new(Url::parse("http://froboz:1234").unwrap());

        assert_eq!(
            Url::parse("http://froboz:1234/ext/plantuml_encoded_string").unwrap(),
            srv.url("ext", "plantuml_encoded_string").unwrap()
        );
    }

    #[test]
    fn test_encode_diagram_source() {
        assert_eq!("SrRGrQsnKt0100==", encode_diagram_source("C --|> D"));
    }

    create_mock! {
        impl ImageDownloader for ImageDownloaderMock (self) {
            expect_download_image("download_image"):
                fn download_image(&self, request_url: &Url) -> Result<Vec<u8>>;
        }
    }

    #[test]
    fn test_render_string() {
        let srv = PlantUMLServer::new(Url::parse("http://froboz").unwrap());

        let mut mock_downloader = ImageDownloaderMock::new();
        mock_downloader
            .expect_download_image()
            .called_once()
            .with(deref(
                Url::parse("http://froboz/svg/SrRGrQsnKt0100==").unwrap(),
            ))
            .returning(|_| Ok(b"the rendered image".to_vec()));

        let img_data = srv
            .render_string("C --|> D", "svg", &mock_downloader)
            .unwrap();

        assert_eq!("the rendered image", String::from_utf8_lossy(&img_data));
    }
}
