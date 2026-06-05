//! Install hook command — install shell integration
use anyhow::Result;

pub fn run_install_hook(_shell: Option<&str>) -> Result<()> {
    println!("🔧 f shell hook");
    println!();
    println!("Add the appropriate section to your shell config:");
    println!();
    println!("── Zsh (~/.zshrc) ──");
    println!();
    println!("# f auto-banner hook");
    println!("autoload -U add-zsh-hook");
    println!("add-zsh-hook chpwd _f_hook");
    println!("_f_hook() {{");
    println!("    command f banner \"$PWD\"");
    println!("    eval \"$(command f env \"$PWD\")\"");
    println!("}}");
    println!("_f_hook  # fire on new shell/tab startup");
    println!();
    println!("── Bash (~/.bashrc) ──");
    println!();
    println!("# f auto-banner hook");
    println!("_f_hook() {{");
    println!("    command f banner \"$PWD\"");
    println!("    eval \"$(command f env \"$PWD\")\"");
    println!("}}");
    println!("PROMPT_COMMAND=\"_f_hook${{PROMPT_COMMAND:+;$PROMPT_COMMAND}}\"");
    println!();
    println!("Then reload: exec zsh   # or: source ~/.bashrc");
    Ok(())
}
