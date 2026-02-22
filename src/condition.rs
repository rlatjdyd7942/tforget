use anyhow::{Result, bail};
use std::collections::HashMap;

pub fn evaluate_condition(condition: &str, vars: &HashMap<String, String>) -> Result<bool> {
    let condition = condition.trim();

    if let Some((var, value)) = parse_contains(condition) {
        let var_value = vars
            .get(var)
            .ok_or_else(|| anyhow::anyhow!("variable '{var}' not found"))?;
        Ok(var_value.split(',').any(|v| v.trim() == value))
    } else if let Some((var, value)) = parse_equals(condition) {
        let var_value = vars
            .get(var)
            .ok_or_else(|| anyhow::anyhow!("variable '{var}' not found"))?;
        Ok(var_value == value)
    } else if let Some((var, value)) = parse_not_equals(condition) {
        let var_value = vars
            .get(var)
            .ok_or_else(|| anyhow::anyhow!("variable '{var}' not found"))?;
        Ok(var_value != value)
    } else {
        bail!("unsupported condition syntax: '{condition}'")
    }
}

fn parse_contains(s: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = s.splitn(2, " contains ").collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), strip_quotes(parts[1].trim())))
    } else {
        None
    }
}

fn parse_equals(s: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = s.splitn(2, " == ").collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), strip_quotes(parts[1].trim())))
    } else {
        None
    }
}

fn parse_not_equals(s: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = s.splitn(2, " != ").collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), strip_quotes(parts[1].trim())))
    } else {
        None
    }
}

fn strip_quotes(s: &str) -> &str {
    s.trim_matches('\'').trim_matches('"')
}
