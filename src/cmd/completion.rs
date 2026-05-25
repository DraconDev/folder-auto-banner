//! Completion command — generate shell completions
use anyhow::Result;
use clap::Command;
use clap_complete::{Generator, shells, generate};

pub fn run_completion(shell: &str) -> Result<()> {
    let mut cmd = Command::new("fm")
        .version(clap::crate_version!())
        .about("cfm — Contextual File Manager");
    
    // Build the command structure (same as in cli/mod.rs)
    build_command(&mut cmd);
    
    match shell.to_lowercase().as_str() {
        "bash" => {
            generate(shells::Bash, &mut cmd, "fm", &mut std::io::stdout());
        }
        "zsh" => {
            generate(shells::Zsh, &mut cmd, "fm", &mut std::io::stdout());
        }
        "fish" => {
            generate(shells::Fish, &mut cmd, "fm", &mut std::io::stdout());
        }
        "powershell" | "ps" => {
            generate(shells::PowerShell, &mut cmd, "fm", &mut std::io::stdout());
        }
        "elvish" => {
            generate(shells::Elvish, &mut cmd, "fm", &mut std::io::stdout());
        }
        _ => {
            println!("❌ Unknown shell: {}", shell);
            println!("Supported shells: bash, zsh, fish, powershell, elvish");
            println!();
            println!("Usage:");
            println!("  fm completion bash >> ~/.bashrc");
            println!("  fm completion zsh >> ~/.zshrc");
            println!("  fm completion fish > ~/.config/fish/completions/fm.fish");
        }
    }
    Ok(())
}

/// Build the command structure (simplified version for completions)
fn build_command(cmd: &mut Command) {
    use clap::{Arg, Command as C};
    
    cmd.subcommand(C::new("banner")
        .about("Print the contextual directory dashboard"))
      .subcommand(C::new("env")
        .about("Output shell aliases for current project type"))
      .subcommand(C::new("yank")
        .about("Yank (copy) files to the clipboard")
        .arg(Arg::new("paths").last(true)))
      .subcommand(C::new("paste")
        .about("Paste (copy) yanked files to current directory")
        .arg(Arg::new("move_files").long("move-files"))
        .arg(Arg::new("overwrite").long("overwrite")))
      .subcommand(C::new("mv")
        .about("Move files"))
      .subcommand(C::new("cp")
        .about("Copy files"))
      .subcommand(C::new("rm")
        .about("Remove files"))
      .subcommand(C::new("trash")
        .about("Move files to trash"))
      .subcommand(C::new("open")
        .about("Open files with default application"))
      .subcommand(C::new("do")
        .about("Act on piped file paths"))
      .subcommand(C::new("stats")
        .about("Deep directory synthesis chart"))
      .subcommand(C::new("pin")
        .about("Pin a directory"))
      .subcommand(C::new("jump")
        .about("Jump to a pinned directory"))
      .subcommand(C::new("pins")
        .about("List all pins"))
      .subcommand(C::new("unpin")
        .about("Unpin a directory"))
      .subcommand(C::new("root")
        .about("Jump to git repo root"))
      .subcommand(C::new("save-session")
        .about("Save current session"))
      .subcommand(C::new("load-session")
        .about("Load a saved session"))
      .subcommand(C::new("sessions")
        .about("List all sessions"))
      .subcommand(C::new("delete-session")
        .about("Delete a session"))
      .subcommand(C::new("diff")
        .about("Compare two directories"))
      .subcommand(C::new("install-hook")
        .about("Install shell hooks"))
      .subcommand(C::new("completion")
        .about("Generate shell completions"))
      .subcommand(C::new("config")
        .about("Edit configuration"));
}