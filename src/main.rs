use clap::Parser;
use tforge::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, ai } => {
            println!("Creating project: {name}");
            if let Some(desc) = ai {
                println!("AI mode: {desc}");
            }
        }
        Commands::List => {
            println!("Available templates:");
            println!("  (none bundled yet)");
        }
        Commands::Search { query } => {
            println!("Searching for: {query}");
        }
        Commands::Add { url } => {
            println!("Adding template from: {url}");
        }
        Commands::Resume => {
            println!("Resuming...");
        }
        Commands::Status => {
            println!("Status: no active project");
        }
        Commands::Update => {
            println!("Updating registry...");
        }
        Commands::Config { target, show } => {
            if show {
                println!("Current {target} config: (none)");
            } else {
                println!("Configure {target}...");
            }
        }
    }
}
