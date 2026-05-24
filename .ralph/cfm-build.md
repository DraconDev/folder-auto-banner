# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool, following the todo.md phases.

## Progress (Iteration 2)

### ✅ Phase 0: Project Foundation — DONE
- `cargo new cfm` ✓
- Configure `Cargo.toml` with binary name `fm` ✓
- Set up workspace structure ✓
- Add all dependencies ✓
- `justfile` ✓

### ✅ Phase 1: Core Architecture — DONE
- `is_terminal()` check utility (via `atty` crate) ✓
- Dual-output mode (rich/raw/JSON) ✓
- Data directory setup ✓
- State persistence types (clipboard, pins, sessions, config) ✓
- Error handling with anyhow ✓

### ✅ Phase 2: The Banner — DONE (working!)
- Project type detection (Rust/Node/Python/Go/Ruby/Java/C++) ✓
- Directory metadata gathering ✓
- Git integration (branch, dirty state) ✓
- Smart truncation (top 8 items) ✓
- File type icons (📂📄) ✓
- `--raw` and `--json` flags ✓
- **FIXED**: TTY detection using both stdin and stdout ✓
- **FIXED**: Unicode char boundary panic ✓
- **FIXED**: Box drawing header char-safe slicing ✓

### ✅ Phase 3: Shell Integration — STUBS DONE
- `fm env` command works ✓
- `fm install-hook` command (stub) ✓

### ✅ Phase 4: Context-Aware Environment — DONE
- Project type detection ✓
- Alias generation (Rust/Node/Python/Go/Generic) ✓

### ⏳ Phase 5-11: STUBS (need full implementation)
- All commands exist as scaffolds
- Need to implement actual logic and state persistence

## Status

The banner is working! When run directly in a terminal (via `script` or actual terminal), it shows the rich table. When piped, it shows raw paths. JSON mode works.

**Key commands working:**
```bash
fm                    # → Rich banner in terminal, raw in pipe
fm --json            # → JSON output
fm env               # → Aliases (cargo run, npm run dev, etc.)
fm install-hook      # → Shows hook script
```

## Next Steps (Priority Order)

1. **Fix TTY detection for direct execution** — The `atty` crate returns false when run via `./target/debug/fm` directly, but works via `script`. This is a known limitation; consider using `is-terminal` crate or checking `TERM` env var.

2. **Implement state persistence** — Wire up clipboard, pins, sessions to actual JSON files

3. **Implement yank/paste** — Real copy/move with progress bars

4. **Install hook** — Actually write to shell config files

5. **Polish banner** — Better formatting, fix size display (shows full precision)

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README