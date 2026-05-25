# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2024-05-25

### Added
- Complete CLI with 28 commands
- Rich terminal output with Unicode icons
- TTY detection (rich/raw/JSON modes)
- State persistence (~/.local/share/cfm/)
- Shell integration hooks
- Shell completions (bash, zsh, fish, powershell, elvish)
- GitHub Actions CI/CD

### Commands
- **Banner**: `fm`, `fm banner`, `fm banner --json`, `fm banner --raw`
- **Env**: `fm env`
- **File ops**: `fm mv`, `fm cp`, `fm rm`, `fm trash`
- **Clipboard**: `fm yank`, `fm paste`, `fm clipboard`
- **Pins**: `fm pin`, `fm pins`, `fm jump`, `fm unpin`, `fm root`
- **Utils**: `fm open`, `fm do`, `fm stats`, `fm diff`
- **Sessions**: `fm save-session`, `fm load-session`, `fm sessions`, `fm delete-session`
- **Shell**: `fm install-hook`, `fm completion`, `fm config`

## [0.1.0] - 2024-05-24

### Added
- Initial release
- Basic banner display
- Project type detection
- Git integration