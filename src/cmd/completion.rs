//! Completion command — generate shell completions
use anyhow::Result;
use clap::Command;
use clap_complete::{Generator, shells, generate};

pub fn run_completion(shell: &str) -> Result<()> {
    let mut cmd = Command::new("fm")
        .version(env!("CARGO_PKG_VERSION"))
        .about("cfm — Contextual File Manager")
        .subcommand(Command::new("banner").about("Print the contextual directory dashboard"))
        .subcommand(Command::new("env").about("Output shell aliases for current project type"))
        .subcommand({
            Command::new("yank").about("Yank (copy) files to the clipboard")
                .arg(clap::Arg::new("paths").required(true).num_args(1..))
        })
        .subcommand({
            Command::new("paste").about("Paste (copy) yanked files to current directory")
                .arg(clap::Arg::new("move_files").long("move-files"))
                .arg(clap::Arg::new("overwrite").long("overwrite"))
        })
        .subcommand({
            Command::new("mv").about("Move files")
                .arg(clap::Arg::new("sources").required(true).num_args(1..))
        })
        .subcommand(Command::new("cp").about("Copy files"))
        .subcommand(Command::new("rm").about("Remove files"))
        .subcommand(Command::new("trash").about("Move files to trash"))
        .subcommand(Command::new("open").about("Open files with default application"))
        .subcommand(Command::new("do").about("Act on piped file paths"))
        .subcommand(Command::new("stats").about("Deep directory synthesis chart"))
        .subcommand({
            Command::new("pin").about("Pin a directory")
                .arg(clap::Arg::new("name").required(true))
        })
        .subcommand({
            Command::new("jump").about("Jump to a pinned directory")
                .arg(clap::Arg::new("name").required(true))
        })
        .subcommand(Command::new("pins").about("List all pins"))
        .subcommand(Command::new("root").about("Jump to git repo root"))
        .subcommand({
            Command::new("save-session").about("Save current session")
                .arg(clap::Arg::new("name").required(true))
        })
        .subcommand({
            Command::new("load-session").about("Load a saved session")
                .arg(clap::Arg::new("name").required(true))
        })
        .subcommand(Command::new("sessions").about("List all sessions"))
        .subcommand({
            Command::new("delete-session").about("Delete a session")
                .arg(clap::Arg::new("name").required(true))
        })
        .subcommand(Command::new("diff").about("Compare two directories"))
        .subcommand(Command::new("install-hook").about("Install shell hooks"))
        .subcommand({
            Command::new("completion").about("Generate shell completions")
                .arg(clap::Arg::new("shell").required(true))
        })
        .subcommand(Command::new("config").about("Edit configuration"));
    
    match shell.to_lowercase().as_str() {
        "bash" => generate(shells::Bash, &mut cmd, "fm", &mut std::io::stdout()),
        "zsh" => generate(shells::Zsh, &mut cmd, "fm", &mut std::io::stdout()),
        "fish" => generate(shells::Fish, &mut cmd, "fm", &mut std::io::stdout()),
        "powershell" | "ps" => generate(shells::PowerShell, &mut cmd, "fm", &mut std::io::stdout()),
        "elvish" => generate(shells::Elvish, &mut cmd, "fm", &mut std::io::stdout()),
        _ => {
            println!("❌ Unknown shell: {}", shell);
            println!("Supported shells: bash, zsh, fish, powershell, elvish");
        }
    }
    Ok(())
}