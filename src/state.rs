use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PipelineState {
    steps: HashMap<String, HashMap<usize, StepStateEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum StepStateEntry {
    Completed,
    Failed(String),
}

#[derive(Debug, PartialEq)]
pub enum StepState {
    Pending,
    Completed,
    Failed(String),
}

impl PipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let state: PipelineState = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn mark_completed(&mut self, template: &str, step_idx: usize) {
        self.steps
            .entry(template.to_string())
            .or_default()
            .insert(step_idx, StepStateEntry::Completed);
    }

    pub fn mark_failed(&mut self, template: &str, step_idx: usize, error: &str) {
        self.steps
            .entry(template.to_string())
            .or_default()
            .insert(step_idx, StepStateEntry::Failed(error.to_string()));
    }

    pub fn get(&self, template: &str, step_idx: usize) -> StepState {
        self.steps
            .get(template)
            .and_then(|s| s.get(&step_idx))
            .map(|e| match e {
                StepStateEntry::Completed => StepState::Completed,
                StepStateEntry::Failed(msg) => StepState::Failed(msg.clone()),
            })
            .unwrap_or(StepState::Pending)
    }
}
