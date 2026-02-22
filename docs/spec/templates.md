# Template System

## Manifest Format

Templates live in `templates/<name>/template.toml`.

```toml
[template]
name = "flutter-app"
description = "Flutter mobile application"
category = "mobile"
provider = "command"

[dependencies]
required_tools = ["flutter"]
requires_templates = []

[parameters]
org = { type = "string", prompt = "Organization name", default = "com.example" }
platforms = { type = "multi-select", prompt = "Target platforms", options = ["ios", "android", "web"], default = ["ios", "android"] }

[[steps]]
type = "command"
command = "flutter create --org {{org}} --platforms {{platforms|join(',')}} {{project_name}}"

[[steps]]
type = "bundled"
action = "overlay"
source = "files/"

[[steps]]
type = "command"
command = "flutter pub get"
working_dir = "{{project_name}}"
```

Key sections:
- `[template]` — name, description, category, provider
- `[dependencies]` — required_tools, requires_templates
- `[parameters]` — user-configurable values (string, multi-select, with defaults)
- `[[steps]]` — ordered actions with optional `condition` and `check` fields

## Providers

1. **Bundled** — static files embedded in the binary via rust-embed
2. **Git** — template repos cloned at runtime
3. **Command** — delegates to existing CLIs (flutter create, npx create-react-app, etc.)

### Dependency Handling

CLI checks if required tools exist, prints install instructions if missing. Does not auto-install.

## Composability

Templates compose via `requires_templates` and conditional steps.

Example: `firebase-flutter` requires `flutter-app` and only appears when Flutter is selected.

```toml
[template]
name = "firebase-flutter"
description = "Firebase setup for Flutter"
category = "integration"
provider = "command"

[dependencies]
required_tools = ["firebase", "flutterfire"]
requires_templates = ["flutter-app"]

[parameters.services]
type = "multi-select"
prompt = "Which Firebase services?"
options = ["crashlytics", "auth", "firestore", "cloud-messaging", "analytics"]
default = ["crashlytics", "analytics"]

[[steps]]
type = "command"
command = "firebase projects:create {{firebase_project_id}}"

[[steps]]
type = "command"
command = "flutterfire configure --project={{firebase_project_id}}"
working_dir = "{{project_name}}"

[[steps]]
type = "command"
condition = "services contains 'crashlytics'"
command = "flutter pub add firebase_crashlytics"
working_dir = "{{project_name}}"
```

## Parameter Sharing

Parameters set in one template (e.g., `gcp_project_id` in `gcp-project`) are automatically available to downstream templates that depend on it.

## Registry

- **Bundled** — ship with binary (flutter-app, axum-server, gcp-project, firebase-flutter)
- **Community** — central `registry.toml` on GitHub pointing to git repos
- **Custom** — `tforge add <git-url>` to add any template

Templates cached at `~/.config/tforge/templates/`.

## Output Structure

```
my-app/
├── tforge.toml          ← recipe manifest (reproducible)
├── app/                 ← Flutter project
├── server/              ← Axum project
└── deploy/              ← deployment configs
```

`tforge.toml` records all selections and parameters, enabling `tforge status` and `tforge rerun`.
