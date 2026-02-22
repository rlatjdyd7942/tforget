use crate::condition::evaluate_condition;
use crate::executor::{execute_step, StepContext, StepResult};
use crate::renderer::Renderer;
use crate::resolver::resolve_order;
use crate::types::TemplateManifest;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Engine {
    project_dir: PathBuf,
    renderer: Renderer,
}

impl Engine {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            project_dir,
            renderer: Renderer::new(),
        }
    }

    pub fn run(&self, templates: &[TemplateManifest], vars: &HashMap<String, String>) -> Result<()> {
        let order = resolve_order(templates)?;

        let template_map: HashMap<&str, &TemplateManifest> = templates
            .iter()
            .map(|t| (t.template.name.as_str(), t))
            .collect();

        for name in &order {
            let tmpl = template_map
                .get(name.as_str())
                .ok_or_else(|| anyhow::anyhow!("template '{name}' not found in map"))?;

            for (i, step) in tmpl.steps.iter().enumerate() {
                // Check condition
                if let Some(cond) = &step.condition {
                    let rendered_cond = self.renderer.render_string(cond, vars)
                        .with_context(|| format!("[{name}] step {}: failed to render condition", i + 1))?;
                    if !evaluate_condition(&rendered_cond, vars)? {
                        continue;
                    }
                }

                // Render step fields
                let mut rendered_step = step.clone();
                if let Some(cmd) = &step.command {
                    rendered_step.command = Some(
                        self.renderer
                            .render_string(cmd, vars)
                            .with_context(|| format!("[{name}] step {}: failed to render command", i + 1))?,
                    );
                }
                if let Some(wd) = &step.working_dir {
                    rendered_step.working_dir = Some(self.renderer.render_string(wd, vars)?);
                }
                if let Some(check) = &step.check {
                    rendered_step.check = Some(self.renderer.render_string(check, vars)?);
                }

                let ctx = StepContext {
                    project_dir: self.project_dir.clone(),
                    vars: vars.clone(),
                };

                match execute_step(&rendered_step, &ctx)
                    .with_context(|| format!("[{name}] step {} failed", i + 1))?
                {
                    StepResult::Executed => {}
                    StepResult::Skipped => {}
                }
            }
        }

        Ok(())
    }
}
