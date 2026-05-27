//! Install hook command — install shell integration
use anyhow::Result;

pub fn run_install_hook(_shell: Option<&str>) -> Result<()> {
    println!("🔧 cfm shell hook");
    println!();
    println!("Add the appropriate section to your shell config:");
    println!();
    println!("── Zsh (~/.zshrc) ──");
    println!();
    println!("# cfm auto-banner hook");
    println!("autoload -U add-zsh-hook");
    println!("add-zsh-hook chpwd _cfm_hook");
    println!("_cfm_hook() {{");
    println!("    command fm banner \"$PWD\"");
    println!("    eval \"$(command fm env \"$PWD\")\"");
    println!("}}");
    println!("_cfm_hook  # fire on new shell/tab startup");
    println!();
    println!("── Bash (~/.bashrc) ──");
    println!();
    println!("# cfm auto-banner hook");
    println!("_cfm_hook() {{");
    println!("    command fm banner \"$PWD\"");
    println!("    eval \"$(command fm env \"$PWD\")\"");
    println!("}}");
    println!("PROMPT_COMMAND=\"_cfm_hook${{PROMPT_COMMAND:+;$PROMPT_COMMAND}}\"");
    println!();
    println!("Then reload: exec zsh   # or: source ~/.bashrc");
    Ok(())
}
