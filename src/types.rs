use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct TemplateManifest {
    pub template: TemplateInfo,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default)]
    pub parameters: HashMap<String, ParamDef>,
    #[serde(default)]
    pub steps: Vec<StepDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub provider: Provider,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Bundled,
    Git,
    Command,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Dependencies {
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub requires_templates: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParamDef {
    #[serde(rename = "type")]
    pub param_type: ParamType,
    pub prompt: String,
    #[serde(default)]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub when: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ParamType {
    String,
    Select,
    MultiSelect,
    Bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StepDef {
    #[serde(rename = "type")]
    pub step_type: String,
    pub command: Option<String>,
    pub condition: Option<String>,
    pub check: Option<String>,
    pub working_dir: Option<String>,
    pub action: Option<String>,
    pub source: Option<String>,
    pub url: Option<String>,
}
