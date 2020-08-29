use failure::Error;
use std::path::PathBuf;

pub trait PlantUMLBackend {
    ///Render a PlantUML string to file and return the diagram URL path to this
    ///file (as a String) for use in a link.
    /// # Arguments
    /// * `plantuml_code` - The present source of the code block
    /// * `output_file` - The path to the file to save the image to
    fn render_from_string(
        &self,
        plantuml_code: &String,
        output_file: &PathBuf,
    ) -> Result<(), Error>;
}
