use anyhow::Result;

pub trait PlantUMLBackend {
    /// Render a PlantUML string to file and return the diagram URL path to this
    /// file (as a String) for use in a link.
    /// # Arguments
    /// * `plantuml_code` - The present source of the code block
    /// * `image_format` - The PlantUML image output format (see -t command line
    ///   option of PlantUML)
    fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>>;
}
