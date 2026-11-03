use crate::backend::Backend;
use anyhow::{Result, bail};

/// Backend using the built-in pure Rust plantuml-rs renderer
pub struct PlantUMLRs;

impl Backend for PlantUMLRs {
    fn render_from_string(&self, plantuml_code: &str, image_format: &str) -> Result<Vec<u8>> {
        if image_format != "svg" {
            bail!(
                "The plantuml-rs backend only supports the svg image format (got '{image_format}')"
            );
        }

        plantuml_core::render(plantuml_code)
            .map(String::into_bytes)
            .map_err(|e| anyhow::format_err!("plantuml-rs failed to render diagram ({e})"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_svg() {
        let svg = PlantUMLRs
            .render_from_string("@startuml\nA -> B\n@enduml", "svg")
            .unwrap();
        assert!(String::from_utf8(svg).unwrap().contains("<svg"));
    }

    #[test]
    fn test_unsupported_format() {
        assert!(
            PlantUMLRs
                .render_from_string("@startuml\nA -> B\n@enduml", "png")
                .is_err()
        );
    }
}
