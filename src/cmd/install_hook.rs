//! Install hook command — install shell integration
use anyhow::Result;

pub fn run_install_hook(_shell: Option<&str>) -> Result<()> {
    println!("🔧 fab shell hook");
    println!();
    println!("Add the appropriate section to your shell config:");
    println!();
    println!("── Zsh (~/.zshrc) ──");
    println!();
    println!("# fab auto-banner hook");
    println!("autoload -U add-zsh-hook");
    println!("add-zsh-hook chpwd _fab_hook");
    println!("_fab_hook() {{");
    println!("    command fm banner \"$PWD\"");
    println!("    eval \"$(command fm env \"$PWD\")\"");
    println!("}}");
    println!("_fab_hook  # fire on new shell/tab startup");
    println!();
    println!("── Bash (~/.bashrc) ──");
    println!();
    println!("# fab auto-banner hook");
    println!("_fab_hook() {{");
    println!("    command fm banner \"$PWD\"");
    println!("    eval \"$(command fm env \"$PWD\")\"");
    println!("}}");
    println!("PROMPT_COMMAND=\"_fab_hook${{PROMPT_COMMAND:+;$PROMPT_COMMAND}}\"");
    println!();
    println!("Then reload: exec zsh   # or: source ~/.bashrc");
    Ok(())
}
