//! Install hook command — install shell integration
use anyhow::Result;

pub fn run_install_hook(_shell: Option<&str>) -> Result<()> {
    println!("🔧 Installing shell hook...");
    println!();
    println!("Add this to your ~/.zshrc or ~/.bashrc:");
    println!();
    println!("# cfm shell integration");
    println!("_cfm_on_directory_change() {{");
    println!("    command fm banner \"$PWD\"");
    println!("    eval \"$(command fm env \"$PWD\")\"");
    println!("}}");
    println!("autoload -U add-zsh-hook");
    println!("add-zash-hook chpwd _cfm_on_directory_change");
    println!();
    println!("💡 Manual installation required for now.");
    Ok(())
}
