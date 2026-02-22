use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tforge",
    version,
    about = "Scaffold multi-stack projects with cloud infrastructure provisioning",
    long_about = "\
tforge is a CLI tool that scaffolds multi-stack projects (Flutter, Axum, \
etc.) with cloud infrastructure provisioning (GCP, Firebase) in a single \
command.\n\n\
Templates are composable TOML manifests. Pick the components you need, \
configure parameters interactively or via LLM, and tforge executes every \
step in dependency order.",
    after_help = "\
EXAMPLES:
  tforge new my-app                  Create a project interactively
  tforge new my-app --ai \"flutter app with firebase\"
                                     Create with LLM assistance
  tforge list                        Show all available templates
  tforge search firebase             Search templates by keyword
  tforge add https://github.com/user/template.git
                                     Add a community template
  tforge resume                      Retry from last failed step
  tforge config llm                  Configure LLM provider"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project from templates
    #[command(
        long_about = "\
Create a new project directory and scaffold it from one or more templates.\n\n\
In interactive mode (default), tforge walks you through template selection \
and parameter configuration. With --ai, a natural language description is \
interpreted by an LLM to select templates automatically.",
        after_help = "\
EXAMPLES:
  tforge new my-app
  tforge new my-app --ai \"flutter app with firebase auth and GCP backend\""
    )]
    New {
        /// Project name (used as directory name)
        name: String,
        /// Natural language project description for LLM-assisted setup
        #[arg(long, value_name = "DESCRIPTION")]
        ai: Option<String>,
    },
    /// Resume execution from the last failed step
    #[command(
        long_about = "\
Resume a previously interrupted or failed project setup. Reads \
.tforge-state.json and retries from the first failure.",
        after_help = "\
EXAMPLES:
  tforge resume"
    )]
    Resume,
    /// Show the current project's execution state
    #[command(
        long_about = "\
Display which templates and steps have been executed, failed, or are \
pending. Reads from .tforge-state.json in the current directory."
    )]
    Status,
    /// List all available templates (bundled and installed)
    #[command(
        long_about = "\
Show all templates including bundled templates shipped with the binary \
and community templates added via `tforge add`."
    )]
    List,
    /// Search the template registry by keyword
    #[command(
        long_about = "\
Search for templates by name, description, or category.",
        after_help = "\
EXAMPLES:
  tforge search firebase
  tforge search gcp"
    )]
    Search {
        /// Search query (matches name, description, and category)
        query: String,
    },
    /// Add a community template from a git repository
    #[command(
        long_about = "\
Clone a template repository into the local cache. The repository must \
contain a template.toml at its root.",
        after_help = "\
EXAMPLES:
  tforge add https://github.com/user/my-template.git"
    )]
    Add {
        /// Git URL of the template repository
        url: String,
    },
    /// Update cached community templates
    #[command(
        long_about = "\
Fetch the latest version of cached community templates via git pull."
    )]
    Update,
    /// Configure tforge settings
    #[command(
        long_about = "\
Manage tforge configuration stored in ~/.config/tforge/config.toml.\n\n\
Use `tforge config llm` to set up an LLM provider for --ai mode.\n\
Use `tforge config reset` to reset all settings.",
        after_help = "\
EXAMPLES:
  tforge config llm                  Set up LLM provider
  tforge config llm --show           Show current LLM config
  tforge config reset                Reset to defaults"
    )]
    Config {
        /// Configuration target (\"llm\" or \"reset\")
        target: String,
        /// Show current configuration without modifying
        #[arg(long)]
        show: bool,
    },
}
