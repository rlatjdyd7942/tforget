# tforge

A Rust CLI tool that scaffolds multi-stack projects with cloud infrastructure provisioning.

## Installation

### From source

```bash
cargo install --path .
```

### Build locally

```bash
cargo build --release
```

## Quick Start

```bash
# Create a project interactively
tforge new my-app

# Create with LLM assistance
tforge new my-app --ai "flutter app with firebase and GCP backend"

# List available templates
tforge list

# Search templates
tforge search firebase
```

## Commands

| Command | Description |
|---------|-------------|
| `tforge new <name>` | Create a new project interactively |
| `tforge new <name> --ai "..."` | Create with LLM-assisted template selection |
| `tforge list` | List all available templates |
| `tforge search <query>` | Search templates by keyword |
| `tforge add <git-url>` | Add a community template |
| `tforge update` | Update cached community templates |
| `tforge resume` | Retry from the last failed step |
| `tforge status` | Show current project execution state |
| `tforge config llm` | Configure LLM provider |
| `tforge config reset` | Reset configuration |

## Bundled Templates

| Template | Category | Description |
|----------|----------|-------------|
| `flutter-app` | mobile | Flutter application scaffold |
| `axum-server` | backend | Axum REST API server |
| `gcp-project` | cloud | Google Cloud project setup |
| `gcp-cloudsql` | cloud | Cloud SQL instance provisioning |
| `gcp-appengine` | cloud | App Engine deployment |
| `firebase-project` | cloud | Firebase project initialization |
| `firebase-flutter` | mobile | Firebase integration for Flutter |

## Configuration

tforge stores its configuration in `~/.config/tforge/config.toml`.

### LLM Setup

```bash
tforge config llm
```

Supported providers: Anthropic, OpenAI, Gemini, Ollama (local).

### Custom Templates

Add community templates from git repositories:

```bash
tforge add https://github.com/user/my-template.git
```

Templates are cached in `~/.config/tforge/templates/`.

## Template Authoring

Templates are TOML manifests. Create a `template.toml`:

```toml
[template]
name = "my-template"
description = "My custom template"
category = "backend"
provider = "command"

[dependencies]
required_tools = ["docker"]

[parameters.db_name]
type = "string"
prompt = "Database name:"
default = "mydb"

[[steps]]
type = "command"
command = "docker run -d --name {{db_name}} postgres"
check = "docker ps --filter name={{db_name}} --format '{{.Names}}'"
```

## Error Recovery

tforge tracks step progress in `.tforge-state.json`. If a step fails:

```bash
tforge status   # See what succeeded and what failed
tforge resume   # Retry from the last failed step
```

## License

MIT
