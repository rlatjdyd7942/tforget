use crate::condition::evaluate_condition;
use crate::executor::{StepContext, StepResult, execute_step};
use crate::renderer::Renderer;
use crate::resolver::resolve_order;
use crate::state::{PipelineState, StepState};
use crate::types::TemplateManifest;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
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

    pub fn run(
        &self,
        templates: &[TemplateManifest],
        vars: &HashMap<String, String>,
    ) -> Result<()> {
        self.run_internal(templates, vars, None, false)
    }

    pub fn run_with_state(
        &self,
        templates: &[TemplateManifest],
        vars: &HashMap<String, String>,
        state_path: &Path,
        resume: bool,
    ) -> Result<()> {
        self.run_internal(templates, vars, Some(state_path), resume)
    }

    fn run_internal(
        &self,
        templates: &[TemplateManifest],
        vars: &HashMap<String, String>,
        state_path: Option<&Path>,
        resume: bool,
    ) -> Result<()> {
        let order = resolve_order(templates)?;
        let mut state = match state_path {
            Some(path) if resume => PipelineState::load(path).with_context(|| {
                format!("failed to load pipeline state from {}", path.display())
            })?,
            _ => PipelineState::new(),
        };

        if state_path.is_some() && !resume {
            save_state_if_needed(&state, state_path)?;
        }

        let template_map: HashMap<&str, &TemplateManifest> = templates
            .iter()
            .map(|t| (t.template.name.as_str(), t))
            .collect();

        for name in &order {
            let tmpl = template_map
                .get(name.as_str())
                .ok_or_else(|| anyhow::anyhow!("template '{name}' not found in map"))?;

            for (i, step) in tmpl.steps.iter().enumerate() {
                if resume && matches!(state.get(name, i), StepState::Completed) {
                    continue;
                }

                // Check condition
                if let Some(cond) = &step.condition {
                    let rendered_cond =
                        self.renderer.render_string(cond, vars).with_context(|| {
                            format!("[{name}] step {}: failed to render condition", i + 1)
                        })?;
                    if !evaluate_condition(&rendered_cond, vars)? {
                        state.mark_completed(name, i);
                        save_state_if_needed(&state, state_path)?;
                        continue;
                    }
                }

                // Render step fields
                let mut rendered_step = step.clone();
                if let Some(cmd) = &step.command {
                    rendered_step.command =
                        Some(self.renderer.render_string(cmd, vars).with_context(|| {
                            format!("[{name}] step {}: failed to render command", i + 1)
                        })?);
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

                match execute_step(&rendered_step, &ctx) {
                    Ok(StepResult::Executed) | Ok(StepResult::Skipped) => {
                        state.mark_completed(name, i);
                        save_state_if_needed(&state, state_path)?;
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        state.mark_failed(name, i, &msg);
                        save_state_if_needed(&state, state_path)?;
                        return Err(err).with_context(|| {
                            format!("[{name}] step {} ({}) failed", i + 1, step.step_type)
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

fn save_state_if_needed(state: &PipelineState, state_path: Option<&Path>) -> Result<()> {
    if let Some(path) = state_path {
        state
            .save(path)
            .with_context(|| format!("failed to save pipeline state to {}", path.display()))?;
    }
    Ok(())
}
