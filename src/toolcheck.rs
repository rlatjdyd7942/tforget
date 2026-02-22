use std::process::Command;

pub enum ToolStatus {
    Found(String),
    NotFound,
}

pub fn check_tool(name: &str) -> ToolStatus {
    match Command::new("which").arg(name).output() {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            ToolStatus::Found(path)
        }
        _ => ToolStatus::NotFound,
    }
}

pub fn install_hint(tool: &str) -> &str {
    match tool {
        "flutter" => "Install Flutter: https://docs.flutter.dev/get-started/install",
        "gcloud" => "Install gcloud CLI: https://cloud.google.com/sdk/docs/install",
        "firebase" => "Install Firebase CLI: npm install -g firebase-tools",
        "flutterfire" => "Install FlutterFire CLI: dart pub global activate flutterfire_cli",
        "node" | "npm" | "npx" => "Install Node.js: https://nodejs.org/",
        "cargo" => "Install Rust: https://rustup.rs/",
        "docker" => "Install Docker: https://docs.docker.com/get-docker/",
        "terraform" => "Install Terraform: https://developer.hashicorp.com/terraform/install",
        _ => "Please install this tool and try again",
    }
}
