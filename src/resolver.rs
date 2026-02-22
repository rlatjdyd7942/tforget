use crate::types::TemplateManifest;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

pub fn resolve_order(templates: &[TemplateManifest]) -> Result<Vec<String>> {
    let names: HashSet<&str> = templates.iter().map(|t| t.template.name.as_str()).collect();
    let deps: HashMap<&str, Vec<&str>> = templates
        .iter()
        .map(|t| {
            let name = t.template.name.as_str();
            let reqs: Vec<&str> = t
                .dependencies
                .requires_templates
                .iter()
                .filter(|r| names.contains(r.as_str()))
                .map(|r| r.as_str())
                .collect();
            (name, reqs)
        })
        .collect();

    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    for name in &names {
        if !visited.contains(name) {
            visit(name, &deps, &mut visited, &mut in_stack, &mut order)?;
        }
    }

    Ok(order)
}

fn visit<'a>(
    node: &'a str,
    deps: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
    order: &mut Vec<String>,
) -> Result<()> {
    if in_stack.contains(node) {
        bail!("circular dependency detected involving '{node}'");
    }
    if visited.contains(node) {
        return Ok(());
    }

    in_stack.insert(node);

    if let Some(node_deps) = deps.get(node) {
        for dep in node_deps {
            visit(dep, deps, visited, in_stack, order)?;
        }
    }

    in_stack.remove(node);
    visited.insert(node);
    order.push(node.to_string());
    Ok(())
}
