use crate::types::StepDef;
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub struct StepContext {
    pub project_dir: PathBuf,
    pub vars: HashMap<String, String>,
}

pub enum StepResult {
    Executed,
    Skipped,
}

pub fn execute_step(step: &StepDef, ctx: &StepContext) -> Result<StepResult> {
    let working_dir = match &step.working_dir {
        Some(dir) => ctx.project_dir.join(dir),
        None => ctx.project_dir.clone(),
    };

    // Run idempotency check if present
    if let Some(check_cmd) = &step.check {
        let status = Command::new("sh")
            .arg("-c")
            .arg(check_cmd)
            .current_dir(&working_dir)
            .status()
            .context("failed to run check command")?;
        if status.success() {
            return Ok(StepResult::Skipped);
        }
    }

    match step.step_type.as_str() {
        "command" => {
            let cmd = step
                .command
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("command step missing 'command' field"))?;
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(&working_dir)
                .output()
                .with_context(|| format!("failed to execute: {cmd}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("command failed: {cmd}\n{stderr}");
            }
            Ok(StepResult::Executed)
        }
        "bundled" => {
            // Will be implemented when we add the bundled file provider
            Ok(StepResult::Executed)
        }
        "git" => {
            let url = step
                .url
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("git step missing 'url' field"))?;
            let output = Command::new("git")
                .args(["clone", "--depth", "1", url])
                .current_dir(&working_dir)
                .output()
                .with_context(|| format!("failed to clone: {url}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("git clone failed: {url}\n{stderr}");
            }
            Ok(StepResult::Executed)
        }
        other => bail!("unknown step type: {other}"),
    }
}
