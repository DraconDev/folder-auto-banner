//! Environment command — inject project-specific aliases
//!
//! Outputs shell aliases based on project type.

use anyhow::Result;
use std::path::Path;

use crate::fs::ProjectType;

/// Run env command
pub fn run_env(path: Option<&Path>, _format: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let path = path.unwrap_or(cwd.as_path());
    let project_type = ProjectType::detect(path);

    let aliases = generate_aliases(&project_type);

    for alias in aliases {
        println!("{}", alias);
    }

    Ok(())
}

fn generate_aliases(project_type: &ProjectType) -> Vec<String> {
    let mut aliases = Vec::new();

    match project_type {
        ProjectType::Rust => {
            aliases.push("alias run='cargo run'".to_string());
            aliases.push("alias test='cargo test'".to_string());
            aliases.push("alias build='cargo build'".to_string());
            aliases.push("alias check='cargo check'".to_string());
            aliases.push("alias clippy='cargo clippy -- -W clippy::all'".to_string());
            aliases.push("alias cfm_clean='cargo clean && fm banner'".to_string());
        }
        ProjectType::Node => {
            aliases.push("alias run='npm run dev'".to_string());
            aliases.push("alias test='npm test'".to_string());
            aliases.push("alias build='npm run build'".to_string());
            aliases.push("alias lint='npm run lint'".to_string());
        }
        ProjectType::Python => {
            aliases.push("alias run='python -m .'".to_string());
            aliases.push("alias test='pytest'".to_string());
            aliases.push("alias venv='source .venv/bin/activate'".to_string());
        }
        ProjectType::Go => {
            aliases.push("alias run='go run .'".to_string());
            aliases.push("alias test='go test ./...'".to_string());
            aliases.push("alias build='go build'".to_string());
        }
        _ => {
            // Generic: check for Makefile
            aliases.push("alias run='make run'".to_string());
            aliases.push("alias build='make'".to_string());
            aliases.push("alias test='make test'".to_string());
        }
    }

    aliases
}
