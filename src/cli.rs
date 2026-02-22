use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tforge", version, about = "Project builder CLI — scaffold multi-stack projects with cloud provisioning")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    New {
        /// Project name
        name: String,
        /// Use LLM to interpret a natural language description
        #[arg(long)]
        ai: Option<String>,
    },
    /// Resume from last failed step
    Resume,
    /// Show current project state
    Status,
    /// List available templates
    List,
    /// Search template registry
    Search {
        /// Search query
        query: String,
    },
    /// Add a community template from a git URL
    Add {
        /// Git URL of the template repository
        url: String,
    },
    /// Update registry and cached templates
    Update,
    /// Configure tforge settings
    Config {
        /// Configuration target (e.g., "llm")
        target: String,
        /// Show current config
        #[arg(long)]
        show: bool,
    },
}
