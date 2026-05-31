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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_aliases_rust() {
        let aliases = generate_aliases(&ProjectType::Rust);
        assert!(aliases.iter().any(|a| a.contains("cargo run")));
        assert!(aliases.iter().any(|a| a.contains("cargo test")));
        assert!(aliases.iter().any(|a| a.contains("cargo build")));
    }

    #[test]
    fn test_generate_aliases_node() {
        let aliases = generate_aliases(&ProjectType::Node);
        assert!(aliases.iter().any(|a| a.contains("npm run dev")));
        assert!(aliases.iter().any(|a| a.contains("npm test")));
    }

    #[test]
    fn test_generate_aliases_python() {
        let aliases = generate_aliases(&ProjectType::Python);
        assert!(aliases.iter().any(|a| a.contains("pytest")));
        assert!(aliases.iter().any(|a| a.contains("python -m")));
    }

    #[test]
    fn test_generate_aliases_go() {
        let aliases = generate_aliases(&ProjectType::Go);
        assert!(aliases.iter().any(|a| a.contains("go run")));
        assert!(aliases.iter().any(|a| a.contains("go test")));
    }

    #[test]
    fn test_generate_aliases_generic() {
        let aliases = generate_aliases(&ProjectType::Generic);
        assert!(aliases.iter().any(|a| a.contains("make run")));
        assert!(aliases.iter().any(|a| a.contains("make")));
    }

    #[test]
    fn test_run_env_returns_ok() {
        // run_env should not panic even if current dir detection fails
        let result = run_env(None, None);
        assert!(result.is_ok());
    }
}
