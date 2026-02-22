use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::config::TforgeConfig;
use crate::registry::Registry;
use crate::types::TemplateManifest;

/// Get the template cache directory (~/.config/tforge/templates/).
pub fn cache_dir() -> PathBuf {
    TforgeConfig::config_dir().join("templates")
}

/// Extract a repository name from a git URL.
fn repo_name_from_url(url: &str) -> Result<String> {
    let trimmed = url.trim_end_matches('/');
    let last_segment = trimmed
        .rsplit('/')
        .next()
        .or_else(|| trimmed.rsplit(':').next())
        .context("cannot extract repo name from URL")?;

    let name = last_segment.trim_end_matches(".git");
    if name.is_empty() {
        bail!("cannot extract repo name from URL: {}", url);
    }
    Ok(name.to_string())
}

/// Add a template from a git URL — clone to cache directory.
pub fn add_template(git_url: &str) -> Result<String> {
    let name = repo_name_from_url(git_url)?;
    let cache = cache_dir();
    std::fs::create_dir_all(&cache)
        .with_context(|| format!("creating cache directory {}", cache.display()))?;

    let dest = cache.join(&name);
    if dest.exists() {
        bail!(
            "template '{}' already exists in cache. Remove it first or run `tforge update`.",
            name
        );
    }

    let output = std::process::Command::new("git")
        .args(["clone", "--depth", "1", git_url])
        .arg(&dest)
        .output()
        .context("failed to run git clone — is git installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed: {}", stderr.trim());
    }

    // Verify template.toml exists in cloned repo
    if !dest.join("template.toml").exists() {
        let _ = std::fs::remove_dir_all(&dest);
        bail!("cloned repository does not contain a template.toml — not a valid tforge template");
    }

    Ok(name)
}

/// Update all cached templates by running `git pull` in each.
pub fn update_templates() -> Result<Vec<String>> {
    let cache = cache_dir();
    if !cache.exists() {
        return Ok(Vec::new());
    }

    let mut updated = Vec::new();
    for entry in std::fs::read_dir(&cache)
        .with_context(|| format!("reading cache directory {}", cache.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() || !path.join(".git").exists() {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let output = std::process::Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(&path)
            .output()
            .with_context(|| format!("running git pull in {}", path.display()))?;

        if output.status.success() {
            updated.push(name);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("warning: git pull failed for '{}': {}", name, stderr.trim());
        }
    }

    Ok(updated)
}

/// List cached remote templates from a specific directory.
pub fn list_cached_templates_in(cache: &Path) -> Result<Vec<TemplateManifest>> {
    if !cache.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    for entry in std::fs::read_dir(cache)
        .with_context(|| format!("reading directory {}", cache.display()))?
    {
        let entry = entry?;
        let template_toml = entry.path().join("template.toml");
        if template_toml.exists() {
            let content = std::fs::read_to_string(&template_toml)
                .with_context(|| format!("reading {}", template_toml.display()))?;
            match toml::from_str::<TemplateManifest>(&content) {
                Ok(manifest) => templates.push(manifest),
                Err(e) => {
                    eprintln!("warning: skipping {}: {}", template_toml.display(), e);
                }
            }
        }
    }

    templates.sort_by(|a, b| a.template.name.cmp(&b.template.name));
    Ok(templates)
}

/// Search templates by name or description (case-insensitive partial match).
pub fn search_templates<'a>(registry: &'a Registry, query: &str) -> Vec<&'a TemplateManifest> {
    let query_lower = query.to_lowercase();
    registry
        .templates()
        .iter()
        .filter(|t| {
            t.template.name.to_lowercase().contains(&query_lower)
                || t.template.description.to_lowercase().contains(&query_lower)
                || t.template.category.to_lowercase().contains(&query_lower)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_name_https() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/my-template.git").unwrap(),
            "my-template"
        );
    }

    #[test]
    fn test_repo_name_no_git_suffix() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/my-template").unwrap(),
            "my-template"
        );
    }

    #[test]
    fn test_repo_name_trailing_slash() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/my-template/").unwrap(),
            "my-template"
        );
    }

    #[test]
    fn test_cache_dir_under_config() {
        let dir = cache_dir();
        assert!(dir.ends_with("tforge/templates"));
    }
}
