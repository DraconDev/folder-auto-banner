//! Completion command — generate shell completions
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Generator, shells, generate};

use crate::cli::Cli;

pub fn run_completion(shell: &str) -> Result<()> {
    let mut cmd = Cli::command();
    
    match shell.to_lowercase().as_str() {
        "bash" => generate_completion::<shells::Bash>(&mut cmd),
        "zsh" => generate_completion::<shells::Zsh>(&mut cmd),
        "fish" => generate_completion::<shells::Fish>(&mut cmd),
        "powershell" | "ps" => generate_completion::<shells::PowerShell>(&mut cmd),
        "elvish" => generate_completion::<shells::Elvish>(&mut cmd),
        _ => {
            println!("❌ Unknown shell: {}", shell);
            println!("Supported shells: bash, zsh, fish, powershell, elvish");
            println!();
            println!("Usage:");
            println!("  fm completion bash >> ~/.bashrc");
            println!("  fm completion zsh >> ~/.zshrc");
            println!("  fm completion fish > ~/.config/fish/completions/fm.fish");
            Ok(())
        }
    }
}

fn generate_completion<G: Generator>(cmd: &mut clap::Command)
where
    Result<(), Box<dyn std::error::Error>>: From<<G as Generator>::Error>,
{
    generate::<G, _>(cmd, "fm", &mut std::io::stdout());
}