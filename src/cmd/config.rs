//! Config command — configuration management
use anyhow::Result;

use crate::state::Config;

pub fn run_config(edit: bool, get: Option<&str>, set: Option<&str>) -> Result<()> {
    if edit {
        let config_path = Config::config_path()?;
        // Ensure config file exists
        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
        }
        // Try to open in $EDITOR
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let status = std::process::Command::new(&editor)
            .arg(&config_path)
            .status();
        match status {
            Ok(s) => {
                if !s.success() {
                    eprintln!("⚠️  Editor exited with status: {}", s);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to open editor: {}", e);
                println!("💡 Config file: {}", config_path.display());
            }
        }
    } else if let Some(key) = get {
        let config = Config::load()?;
        match key {
            "icons" => println!("{}", config.icons),
            "colors" => println!("{}", config.colors),
            "compact" => println!("{}", config.compact),
            "max_display_items" => println!("{}", config.max_display_items),
            "mutators" => println!("{:?}", config.mutators),
            "disabled_dirs" => println!("{:?}", config.disabled_dirs),
            _ => {
                eprintln!("❌ Unknown config key: {}", key);
                println!("Available keys: icons, colors, compact, max_display_items, mutators, disabled_dirs");
            }
        }
    } else if let Some(kv) = set {
        let mut config = Config::load()?;
        let (key, value) = kv
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("Invalid format. Use KEY=VALUE"))?;
        match key {
            "icons" => config.icons = value.parse()?,
            "colors" => config.colors = value.parse()?,
            "compact" => config.compact = value.parse()?,
            "max_display_items" => config.max_display_items = value.parse()?,
            "mutators" => {
                config.mutators = value.split(',').map(|s| s.trim().to_string()).collect()
            }
            "disabled_dirs" => {
                config.disabled_dirs = value.split(',').map(|s| s.trim().to_string()).collect()
            }
            _ => {
                eprintln!("❌ Unknown config key: {}", key);
                println!("Available keys: icons, colors, compact, max_display_items, mutators, disabled_dirs");
                return Ok(());
            }
        }
        config.save()?;
        println!("✅ Set {} = {}", key, value);
    } else {
        let config = Config::load()?;
        println!("⚙️  Configuration:");
        println!("  icons: {}", config.icons);
        println!("  colors: {}", config.colors);
        println!("  compact: {}", config.compact);
        println!("  max_display_items: {}", config.max_display_items);
        println!("  mutators: {:?}", config.mutators);
        println!("  disabled_dirs: {:?}", config.disabled_dirs);
        println!();
        let config_path = Config::config_path()?;
        println!("💡 Config file: {}", config_path.display());
        println!("💡 Use 'fm config --edit' to open in $EDITOR");
        println!("💡 Use 'fm config --set key=value' to change a setting");
    }
    Ok(())
}
