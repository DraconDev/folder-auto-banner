//! Completion command — generate shell completions
use anyhow::Result;

pub fn run_completion(shell: &str) -> Result<()> {
    println!("🔧 Generating {} completions...", shell);
    
    match shell.to_lowercase().as_str() {
        "bash" => println!("💡 Use: fm completion bash >> ~/.bashrc"),
        "zsh" => println!("💡 Use: fm completion zsh >> ~/.zshrc"),
        "fish" => println!("💡 Use: fm completion fish > ~/.config/fish/completions/fm.fish"),
        _ => println!("Supported shells: bash, zsh, fish"),
    }
    
    println!();
    println!("# Add to your shell config to enable completions:");
    println!("mkdir -p ~/.config/fm");
    println!("fm completion $(echo $SHELL | xargs basename) >> ~/.$(basename $SHELL)rc");
    
    Ok(())
}
