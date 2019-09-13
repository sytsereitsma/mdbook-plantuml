use failure::Error;
use plantuml_backend::PlantUMLBackend;
use std::path::PathBuf;
use url::Url;

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
}

impl PlantUMLBackend for PlantUMLServer {
    fn render_from_string(&self, plantuml_code: &String) -> Result<String, Error> {
        bail!("Not implemented")
    }
}
