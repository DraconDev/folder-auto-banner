# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool, following the todo.md phases.

## Goal
A working, testable Rust binary (`fm`) that implements the "zero-hostage ephemeral CLI" philosophy.

## Phases (in order)

### Phase 0: Project Foundation
- `cargo new cfm` inside `/home/dracon/Dev/cli-file-manager/`
- Configure `Cargo.toml` with binary name `fm`
- Set up workspace structure: `src/main.rs`, `src/cli.rs`, `src/banner.rs`, `src/commands/`, `src/shell/`, `src/git.rs`, `src/fs.rs`, `src/state.rs`
- Add dependencies: `clap`, `comfy-table`, `console`, `ignore`, `git2`, `serde`, `serde_json`, `directories`, `indicatif`, `clap_complete`, `humansize`, `chrono`
- `justfile` for dev tasks
- `.gitignore`

### Phase 1: Core Architecture & Constraints
- `is_terminal()` check utility
- Dual-output mode (rich / raw / JSON)
- Data directory setup via `directories` crate
- State persistence (clipboard.json, pins.json)
- Error handling with anyhow

### Phase 2: The Banner (`fm banner`)
- Project type detection
- Directory metadata gathering
- Git integration (branch, dirty state, ahead/behind)
- Smart truncation (top 8 items)
- File type icons
- `--raw` and `--json` flags

### Phase 3: Shell Integration
- `fm env` command
- `fm install-hook` command (Zsh/Bash)
- Shell wrapper function
- Bypass mechanisms (CFM_QUIET, cdq)

### Phase 4: Context-Aware Environment
- Project type detection (Rust, Node, Python, Go)
- Alias generation

### Phase 5: File Operations (`fm mv`)
- Split context dashboard for move operations
- Collision detection

### Phase 6: Ephemeral Clipboard (`fm yank` / `fm paste`)
- Cross-terminal clipboard with JSON state

### Phase 7: Safe File Operations (`fm rm` / `fm trash` / `fm open`)
- Confirmation, dry-run, piped input support

### Phase 8: Smart Piping (`fm do`)
- stdin pipe destination with action mapping

### Phase 9: Directory Stats (`fm stats`)
- Deep synthesis charts

### Phase 10: Spatial Memory (`fm pin` / `fm jump` / `fm root`)
- Bookmark system with shell wrapper

### Phase 11: Session Management (`fm save-session` / `fm load-session`)
- Workspace persistence

### Phase 12: Directory Comparison (`fm diff`)
- Visual diff between directories

### Phase 13: Shell Completions
- `clap_complete` integration

### Phase 14: Configuration
- `~/.config/cfm/config.toml` support

### Phase 15: Polish & Edge Cases
- NO_COLOR, Unicode fallback, cross-platform

### Phase 16: Testing
- Unit tests, integration tests, benchmarks

### Phase 17: Documentation
- README, manifest, comparison with other tools

### Phase 18: Packaging
- crates.io publish, GitHub Actions CI

## Constraints
- NO TUI / alternate screen buffer
- NO background daemons / watchers
- NO interactive prompts (no `inquire`, no raw mode)
- Every command: print once, exit immediately
- Target latency: <5ms for empty dirs, <50ms for 10k files
- Dual output: rich (TTY) vs raw/JSON (piped)

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README