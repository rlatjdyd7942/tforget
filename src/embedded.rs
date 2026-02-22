use anyhow::{Context, Result};
use rust_embed::RustEmbed;

use crate::types::TemplateManifest;

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct TemplateAssets;

/// Load all template manifests from the embedded `templates/` directory.
pub fn load_embedded_templates() -> Result<Vec<TemplateManifest>> {
    let mut templates = Vec::new();

    for path in TemplateAssets::iter() {
        if path.ends_with("template.toml") {
            let file = TemplateAssets::get(&path)
                .with_context(|| format!("reading embedded asset {path}"))?;
            let content = std::str::from_utf8(&file.data)
                .with_context(|| format!("embedded asset {path} is not valid UTF-8"))?;
            let manifest: TemplateManifest = toml::from_str(content)
                .with_context(|| format!("parsing embedded template {path}"))?;
            templates.push(manifest);
        }
    }

    Ok(templates)
}
