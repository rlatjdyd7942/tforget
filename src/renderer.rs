use anyhow::{Context, Result};
use minijinja::{Environment, UndefinedBehavior};
use std::collections::HashMap;

pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        Self { env }
    }

    pub fn render_string(&self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        let tmpl = self
            .env
            .template_from_str(template)
            .context("failed to parse template string")?;
        let result = tmpl.render(vars).context("failed to render template")?;
        Ok(result)
    }
}
