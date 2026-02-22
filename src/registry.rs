use crate::types::TemplateManifest;
use anyhow::{Context, Result};
use std::path::Path;

pub struct Registry {
    templates: Vec<TemplateManifest>,
}

impl Registry {
    /// Load templates from embedded assets (bundled in the binary).
    pub fn from_embedded() -> Result<Self> {
        let templates = crate::embedded::load_embedded_templates()?;
        let mut registry = Self { templates };
        registry
            .templates
            .sort_by(|a, b| a.template.name.cmp(&b.template.name));
        Ok(registry)
    }

    /// Merge another registry into this one. Deduplicates by template name.
    pub fn merge(&mut self, other: Registry) {
        for t in other.templates {
            if !self.templates.iter().any(|e| e.template.name == t.template.name) {
                self.templates.push(t);
            }
        }
        self.templates
            .sort_by(|a, b| a.template.name.cmp(&b.template.name));
    }

    /// Load templates from the cache directory (~/.config/tforge/templates/).
    pub fn from_cache_dir() -> Result<Self> {
        let cache = crate::remote::cache_dir();
        if cache.exists() {
            Self::from_directory(&cache)
        } else {
            Ok(Self {
                templates: Vec::new(),
            })
        }
    }

    pub fn from_directory(path: &Path) -> Result<Self> {
        let mut templates = Vec::new();

        if !path.exists() {
            return Ok(Self { templates });
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let template_toml = entry.path().join("template.toml");
            if template_toml.exists() {
                let content = std::fs::read_to_string(&template_toml)
                    .with_context(|| format!("reading {}", template_toml.display()))?;
                let manifest: TemplateManifest = toml::from_str(&content)
                    .with_context(|| format!("parsing {}", template_toml.display()))?;
                templates.push(manifest);
            }
        }

        templates.sort_by(|a, b| a.template.name.cmp(&b.template.name));
        Ok(Self { templates })
    }

    pub fn templates(&self) -> &[TemplateManifest] {
        &self.templates
    }

    pub fn find(&self, name: &str) -> Option<&TemplateManifest> {
        self.templates.iter().find(|t| t.template.name == name)
    }

    pub fn by_category(&self, category: &str) -> Vec<&TemplateManifest> {
        self.templates
            .iter()
            .filter(|t| t.template.category == category)
            .collect()
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .templates
            .iter()
            .map(|t| t.template.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }
}
