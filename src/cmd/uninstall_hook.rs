//! Uninstall hook command — remove shell integration
use anyhow::Result;

pub fn run_uninstall_hook() -> Result<()> {
    let shell = detect_shell();
    let config_path = get_shell_config_path(&shell);

    if !config_path.exists() {
        println!("❌ Shell config not found: {}", config_path.display());
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let hook_start = "# f auto-banner hook";
    let hook_end = match shell.as_str() {
        "zsh" => "_f_hook  # fire on new shell/tab startup",
        "bash" => "PROMPT_COMMAND=\"_f_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}\"",
        _ => "_f_hook  # fire on new shell/tab startup",
    };

    if !content.contains(hook_start) {
        println!("ℹ️  No f hook found in {}", config_path.display());
        return Ok(());
    }

    // Find and remove the hook block
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip = false;
    let mut found_start = false;

    for line in &lines {
        if line.trim() == hook_start {
            skip = true;
            found_start = true;
            continue;
        }
        if skip && line.trim() == hook_end {
            skip = false;
            continue;
        }
        if !skip {
            new_lines.push(line.to_string());
        }
    }

    if found_start {
        // Also remove the "autoload -U add-zsh-hook" line for zsh
        if shell == "zsh" {
            new_lines.retain(|l| l.trim() != "autoload -U add-zsh-hook");
        }
        // Remove trailing empty lines
        while new_lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
            new_lines.pop();
        }
        let new_content = new_lines.join("\n") + "\n";
        std::fs::write(&config_path, new_content)?;
        println!("✅ Removed f hook from {}", config_path.display());
        println!("💡 Reload your shell: exec {} or source {}", shell, config_path.display());
    } else {
        println!("ℹ️  No f hook found in {}", config_path.display());
    }

    Ok(())
}

fn detect_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "/bin/sh".to_string())
        .split('/')
        .next_back()
        .unwrap_or("sh")
        .to_string()
}

fn get_shell_config_path(shell: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    match shell {
        "zsh" => std::path::PathBuf::from(home).join(".zshrc"),
        "bash" => std::path::PathBuf::from(home).join(".bashrc"),
        "fish" => std::path::PathBuf::from(home).join(".config/fish/config.fish"),
        _ => std::path::PathBuf::from(home).join(".profile"),
    }
}
