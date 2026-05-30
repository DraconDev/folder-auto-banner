//! Do command — act on piped file paths
//!
//! Reads paths from stdin and executes actions based on file types/extensions.
//! Usage: find . -name "*.rs" | fm do [action]
//!
//! Actions:
//!   test     - Run tests for matching files
//!   lint     - Lint the files
//!   format   - Format the files
//!   count    - Count lines
//!   delete   - Move to trash
//!   (custom) - Execute custom command with {} for path

use anyhow::Result;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

pub fn run_do(action: Option<&str>, verbose: bool, dry_run: bool) -> Result<()> {
    // Read paths from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let paths: Vec<PathBuf> = input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .collect();

    if paths.is_empty() {
        println!("📭 No paths received from stdin");
        return Ok(());
    }

    println!("📥 Received {} path(s)", paths.len());

    let action = action.unwrap_or("list");

    if dry_run {
        println!("🔍 Dry run — no actions will be executed");
        println!();
    }

    match action {
        "list" => {
            for path in &paths {
                println!("  {}", path.display());
            }
        }
        "count" => {
            let mut total_lines = 0;
            let mut total_files = 0;

            for path in &paths {
                if path.is_file() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            let lines = content.lines().count();
                            println!("{:6} {}", lines, path.display());
                            total_lines += lines;
                            total_files += 1;
                        }
                        Err(e) => {
                            eprintln!("⚠️  {}: {}", path.display(), e);
                        }
                    }
                }
            }
            println!("\n📊 Total: {} lines in {} files", total_lines, total_files);
        }
        "delete" | "trash" => {
            for path in &paths {
                if path.exists() {
                    if dry_run {
                        println!("  Would trash: {}", path.display());
                    } else {
                        // Use trash directory
                        if let Some(proj_dirs) =
                            directories::ProjectDirs::from("com", "cfm", "cfm")
                        {
                            let trash_dir = proj_dirs.data_dir().join("trash");
                            std::fs::create_dir_all(&trash_dir)?;

                            let id = format!(
                                "{:x}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_nanos()
                            );
                            let dest = trash_dir.join(&id);

                            if let Err(e) = std::fs::rename(path, &dest) {
                                eprintln!("❌ Failed to trash {}: {}", path.display(), e);
                            } else if verbose {
                                println!("🗑️  Trashed: {}", path.display());
                            }
                        }
                    }
                }
            }
            if dry_run {
                println!(
                    "📋 Would trash {} file(s)",
                    paths.iter().filter(|p| p.exists()).count()
                );
            } else {
                println!(
                    "🗑️  Trashed {} file(s)",
                    paths.iter().filter(|p| p.exists()).count()
                );
            }
        }
        "open" => {
            // Open all files (cross-platform)
            for path in &paths {
                if !path.exists() {
                    eprintln!("❌ Not found: {}", path.display());
                    continue;
                }

                if dry_run {
                    println!("  Would open: {}", path.display());
                } else {
                    #[cfg(target_os = "linux")]
                    let _ = Command::new("xdg-open").arg(path).spawn();

                    #[cfg(target_os = "macos")]
                    let _ = Command::new("open").arg(path).spawn();

                    #[cfg(target_os = "windows")]
                    let _ = Command::new("cmd")
                        .args(["/C", "start", "", path.to_str().unwrap_or("")])
                        .spawn();

                    if verbose {
                        println!("✓ Opened: {}", path.display());
                    }
                }
            }
            if dry_run {
                println!("📋 Would open {} file(s)", paths.len());
            } else {
                println!("✅ Opened {} file(s)", paths.len());
            }
        }
        "cat" | "show" => {
            for path in &paths {
                if path.is_file() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            println!("=== {} ===", path.display());
                            println!("{}", content);
                        }
                        Err(e) => {
                            eprintln!("❌ {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        _ => {
            // Custom command: action is the command template
            // Replace {} with path — use shell-style splitting but protect paths
            let cmd_template = action;

            for path in &paths {
                let cmd_str = cmd_template.replace("{}", &path.to_string_lossy());

                if dry_run {
                    println!("  Would run: {}", cmd_str);
                } else {
                    // Split command, but rejoin anything after the first token that looks like
                    // it could be part of a path argument. We use a simple heuristic:
                    // if the template was `cmd {}`, after replacement it's `cmd /path/with spaces`,
                    // so we split into at most 2 parts: the command and the rest.
                    let (cmd_name, args_str) = match cmd_str.split_once(char::is_whitespace) {
                        Some((name, rest)) => (name, Some(rest)),
                        None => (cmd_str.as_str(), None),
                    };

                    let mut cmd = Command::new(cmd_name);
                    // Pass the rest as a single argument (the path) to avoid splitting paths
                    if let Some(rest) = args_str {
                        // Check if there are multiple {} placeholders or other template tokens
                        if cmd_template.matches("{}").count() > 1 {
                            // Multiple placeholders — split on whitespace (user's responsibility)
                            for arg in rest.split_whitespace() {
                                cmd.arg(arg);
                            }
                        } else {
                            // Single {} — treat the rest as one argument
                            cmd.arg(rest);
                        }
                    }

                    if verbose {
                        println!("$ {}", cmd_str);
                    }

                    match cmd.output() {
                        Ok(output) => {
                            if !output.status.success() {
                                eprintln!("❌ Command failed: {}", cmd_str);
                                if !output.stderr.is_empty() {
                                    eprintln!(
                                        "stderr: {}",
                                        String::from_utf8_lossy(&output.stderr)
                                    );
                                }
                            } else if verbose {
                                println!("✅ {}", path.display());
                                if !output.stdout.is_empty() {
                                    print!("{}", String::from_utf8_lossy(&output.stdout));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to run {}: {}", cmd_str, e);
                        }
                    }
                }
            }
            if dry_run {
                println!("📋 Would run {} command(s)", paths.len());
            }
        }
    }

    Ok(())
}
