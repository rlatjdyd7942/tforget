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
command = "flutter create --org {{org}} --platforms {{platforms}} {{project_name}}"
check = "test -f pubspec.yaml"

[[steps]]
type = "command"
command = "flutter pub get"
working_dir = "{{project_name}}"
```

Key sections:
- `[template]` — name, description, category, provider
- `[dependencies]` — required_tools, requires_templates
- `[parameters]` — user-configurable values (string, multi-select, with defaults) and optional prompt conditions
- `[[steps]]` — ordered actions with optional `condition` and `check` fields

### Parameter Types

Supported parameter types:
- `string`
- `select`
- `multi-select`
- `bool`

### Parameter Prompt Semantics

- Parameter prompts are deterministic: keys are processed in ascending lexical order.
- A parameter may define an optional `when` expression to conditionally show the prompt.
- `when` uses the same expression grammar as step `condition` (`==`, `!=`, `contains`).
- `when` expressions are evaluated against currently known variables (shared vars plus earlier prompted params).
- If `when` evaluates to `false`, the parameter is skipped.
- If `when` references an unknown variable, prompt resolution fails with a clear error.

## Step Types

1. **command** — executes a shell command (`sh -c ...`) with optional `working_dir`.
2. **git** — clones `url` via `git clone --depth 1`.
3. **bundled** — accepted by parser/executor for compatibility; currently treated as executed without file overlay/copy behavior.

### Dependency Handling

CLI checks if required tools exist, prints install instructions if missing. Does not auto-install.

## Provider Metadata

`[template].provider` is template metadata (`bundled`, `git`, `command`) used for discovery/authoring context.

- Bundled templates are embedded into the binary from `templates/**/template.toml`.
- Runtime step execution behavior is determined by each `[[steps]].type`.

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

## Bundled Template Contract: `gcp-appengine`

`gcp-appengine` remains a cloud template that depends on `gcp-project`, and adds guided deployment configuration.

Required parameter groups:

- Common:
  - `deploy_target` (`select`): `project-root`, `flutter-app`, `axum-server`, `custom-path`
  - `deploy_target_path` (`string`, `when = "deploy_target == 'custom-path'"`)
  - `appengine_environment` (`select`): `standard`, `flexible`
  - `service` (`string`, default `default`)
  - `version` (`string`, default `v1`)
  - `deploy_now` (`bool`, default `false`)
  - `promote_traffic` (`bool`, default `true`, `when = "deploy_now == 'true'"`)
- Standard environment (`when = "appengine_environment == 'standard'"`):
  - `runtime_standard` (`select`): `python312`, `nodejs20`, `go122`, `java21`
  - `instance_class` (`select`): `F1`, `F2`, `F4`, `F4_1G`
  - `standard_max_instances` (`string`)
- Flexible environment (`when = "appengine_environment == 'flexible'"`):
  - `runtime_flexible` (`select`): `python`, `nodejs`, `go`, `java`, `custom`
  - `flex_cpu` (`string`)
  - `flex_memory_gb` (`string`)
  - `flex_min_instances` (`string`)
  - `flex_max_instances` (`string`)

Target directory resolution:

- `project-root` → invocation directory
- `flutter-app` → `{{project_name}}`
- `axum-server` → `{{project_name}}-server`
- `custom-path` → `{{deploy_target_path}}`

Execution behavior:

- Ensure App Engine API/service initialization is complete for `{{gcp_project_id}}`.
- Generate/update `app.yaml` in the resolved target directory using selected environment settings.
- If `deploy_now == 'true'`, run `gcloud app deploy` from the resolved target directory with selected service/version/promotion settings.
- Steps remain idempotent through `check` commands where possible.

## Registry

- **Bundled** — template manifests shipped in the binary
- **Local override** — `templates/` directory in the current repository/workspace
- **Cached remote** — git repos cloned via `tforge add <git-url>`

Templates cached at `~/.config/tforge/templates/`.

## Output Structure

Execution writes state files in the invocation directory:

- `tforge.toml` — selected templates and parameter values.
- `.tforge-state.json` — per-template/per-step completion and failure state.

Generated project folders/files are defined by template commands (for example Flutter/Axum/GCP/Firebase CLI commands).
`tforge status` and `tforge resume` read the two state files above.
