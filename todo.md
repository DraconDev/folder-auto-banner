# cfm (Contextual File Manager) — Master TODO

> **Binary:** `fm`  
> **Philosophy:** Ephemeral, zero-hostage intelligence layer for the shell. Fire-and-forget. No TUI. No daemon. <5ms latency.

---

---

## 🚀 Progress Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0: Project Foundation | ✅ DONE | All deps, workspace, justfile |
| Phase 1: Core Architecture | ✅ DONE | TTY detection, dual output, state types, error handling |
| Phase 2: Banner | ✅ MVP DONE | Project detection, git status, truncation, --raw/--json |
| Phase 3: Shell Integration | ✅ STUBS | env command works, install/uninstall/hook stubs |
| Phase 4: Context-Aware Env | ✅ DONE | Rust/Node/Python/Go aliases |
| Phase 5-11 | ⏳ STUBS | Commands exist as scaffolds, need full implementation |
| Phase 12-18 | 📋 PENDING | Not started |

**Next:** Implement state persistence (yank/paste), flesh out banner display, add shell hook installation

---

## Phase 0: Project Foundation

- [x] `cargo new cfm` — create Rust project
- [ ] Configure `Cargo.toml` with binary name `fm`
- [ ] Set up workspace structure: `src/main.rs`, `src/cli.rs`, `src/banner.rs`, `src/commands/`, `src/shell/`, `src/git.rs`, `src/fs.rs`, `src/state.rs`
- [ ] Add core dependencies to `Cargo.toml`:
  - [x] `clap` — CLI parsing with subcommands
  - [x] `comfy-table` — rich table output for banner
  - [x] `console` — colors, styles, terminal width detection
  - [ ] `jwalk` or `ignore` — fast parallel directory walking
  - [x] `git2` or `gix` — Git status detection
  - [x] `serde` + `serde_json` — state persistence (clipboard, pins)
  - [x] `directories` — XDG dirs (`~/.local/share/cfm/`)
  - [x] `indicatif` — progress bars for `paste`/`yank` operations
  - [x] `clap_complete` — shell completion generation
  - [ ] `syntect` — syntax highlighting for `fm peek`
  - [x] `humansize` / `byte-unit` — human-readable sizes
  - [x] `chrono` / `time` — file timestamps
  - [ ] `walkdir` (fallback) or stick with `jwalk`
- [x] Set up `justfile` / `Makefile` for common dev tasks
- [x] Add `.gitignore`
- [ ] Create `README.md` skeleton with manifesto

---

## Phase 1: Core Architecture & Constraints

- [x] Implement `is_terminal()` check utility — detect TTY vs pipe for all output
- [x] Implement dual-output mode:
  - [ ] Rich mode: `comfy-table` with colors, borders, icons, progress bars
  - [ ] Raw mode: clean text lines (for piping)
  - [ ] JSON mode: `--json` flag for scriptability
- [x] Implement config/data directory (`~/.local/share/cfm/` via `directories` crate)
- [x] Implement state persistence layer:
  - [x] `~/.local/share/cfm/clipboard.json` — yank/paste state
  - [x] `~/.local/share/cfm/pins.json` — pinned directories
  - [x] `~/.local/share/cfm/history.json` — frecency data (optional v2)
- [x] Implement error handling strategy (anyhow + human-readable messages)
- [x] Implement logging/debug mode (`--debug`, `RUST_LOG`)
- [ ] Performance budget: every command must complete in <10ms for trivial dirs, <50ms for 10k files

---

## Phase 2: The Banner — MVP DONE ✓ (`fm banner [path]`)

> The crown jewel. Auto-triggered on `cd`. Prints a rich dashboard and exits.  
> **Note:** `fm` with no arguments is a shorthand for `fm banner`. There is no separate `fm ls` command — the banner IS the enhanced directory listing. For raw output, use `fm banner --raw` or pipe to another command.

- [x] Detect project type from files (`Cargo.toml` = Rust, `package.json` = Node, `pyproject.toml` = Python, etc.)
- [ ] Gather directory metadata:
  - [x] Total item count (files + dirs)
  - [x] Total size (human-readable)
  - [x] Top-level item listing (files + dirs with counts)
  - [ ] Last modified time summary
- [x] Git integration:
  - [x] Detect if inside Git repo
  - [ ] Show branch name
  - [ ] Show ahead/behind count (`↑2 ↓0`)
  - [x] Show dirty state (`✚3` modified, `?1` untracked)
  - [ ] Show last commit message (truncated)
- [x] Smart truncation:
  - [x] Show top 8 items max
  - [ ] Show `... and N more items. (Use 'ls' to see all)`
  - [ ] Handle very wide terminals vs narrow terminals gracefully
- [x] File type icons / emojis (📂 📄 🦀 ⚙️ 📦 📝)
- [ ] Size visualization (optional): mini bar charts or color-coding by size
- [ ] Contextual footer:
  - [ ] Quick actions available (`run`, `test`, `build`) if detected
  - [ ] Readable metadata (e.g., "Node v20.1 | 📦 npm")
- [x] Implement `--raw` flag: print plain file list for piping
- [x] Implement `--json` flag: structured JSON output
- [x] Terminal width detection + responsive layout

### Banner Mockup Target:
```text
┌─ 📂 ~/Dev/dracon-utilities ──────────────────────── [main ↑2 ✚3] ─┐
│ 🦀 Rust Workspace  │  📦 14.2 MB  │  🕒 4 files changed today     │
│                                                                    │
│ 📂 src/            12 items   📂 dracon-sync/     8 items          │
│ 📂 docs/            4 items   📄 Cargo.toml       1.2 KB           │
│ 📄 README.md        4.5 KB    📄 tarpaulin.toml   800 B            │
│                                                                    │
│ ... and 14 more items. (Use 'ls' to see all)                       │
└────────────────────────────────────────────────────────────────────┘
```

---

## Phase 3: Shell Integration — STUBS DONE

> The magic glue. `fm` relies on shell hooks to feel ambient.

- [x] Implement hidden/internal commands for shell consumption:
  - [x] `fm env [path]` — output shell alias definitions for current project
  - [ ] `fm complete --dir <path>` — output completions for shell TAB integration
- [ ] Create shell hook scripts:
  - [ ] **Zsh hook** (`~/.zshrc` integration):
    - [ ] `chpwd` hook: run `fm banner "$PWD"` + `eval "$(fm env "$PWD")"`
    - [ ] `precmd` hook: smart refresh after mutator commands
    - [ ] List of mutators: `git`, `npm`, `cargo`, `make`, `rm`, `mv`, `cp`, `mkdir`, `touch`
  - [ ] **Bash hook** (`~/.bashrc` integration):
    - [ ] `PROMPT_COMMAND` equivalent
    - [ ] `cd` wrapper or `chpwd` equivalent
  - [ ] **Fish hook** (optional, v2)
- [ ] Implement `fm install-hook` command:
  - [ ] Auto-detect shell (Zsh/Bash/Fish)
  - [ ] Append hook block to shell config (marked with `# cfm start` / `# cfm end`)
  - [ ] Idempotent — don't duplicate on re-run
  - [ ] Backup original config before modifying
- [ ] Implement `fm uninstall-hook` command:
  - [ ] Remove `# cfm start` ... `# cfm end` block from shell config
- [ ] Implement bypass mechanism:
  - [ ] Environment variable `CFM_QUIET=1` to skip banner
  - [ ] Shell alias `cdq` (cd quiet) that bypasses hook
- [ ] **The Shell Wrapper Function** (required for `jump`, `root`, `load-session` to work):
  ```bash
  fm() {
    case "$1" in
      jump)
        local target=$(command fm jump --print-cd "$2")
        [[ -n "$target" ]] && eval "$target" && command fm banner "$PWD"
        ;;
      root)
        local target=$(command fm root --print-cd)
        [[ -n "$target" ]] && eval "$target" && command fm banner "$PWD"
        ;;
      load-session)
        local target=$(command fm load-session --print-cd "$2")
        [[ -n "$target" ]] && eval "$target" && command fm banner "$PWD"
        ;;
      *)
        command fm "$@"
        ;;
    esac
  }
  ```
  - [ ] Install this wrapper via `fm install-hook`
  - [ ] Ensure it doesn't conflict with existing `fm` aliases
- [ ] Document hook performance: must add <5ms to `cd` latency

---

## Phase 4: Context-Aware Environment — DONE ✓ (`fm env`)

> Injects project-specific aliases into the shell on directory change.

- [x] Detect project type and generate aliases:
  - [x] **Rust** (`Cargo.toml`):
    - `run` → `cargo run`
    - `test` → `cargo test`
    - `build` → `cargo build`
    - `check` → `cargo check`
    - `cfm_clean` → `cargo clean && fm banner`
  - [x] **Node.js** (`package.json`):
    - `run` → `npm run dev` (or `npm start` if no dev)
    - `test` → `npm test`
    - `build` → `npm run build`
    - `lint` → `npm run lint` (if exists)
  - [x] **Python** (`pyproject.toml`, `setup.py`, `requirements.txt`):
    - `run` → `python -m ...` (detect entry point)
    - `test` → `pytest`
    - `venv` → `source .venv/bin/activate`
  - [x] **Go** (`go.mod`):
    - `run` → `go run .`
    - `test` → `go test ./...`
    - `build` → `go build`
  - [x] **Generic** (Makefile):
    - `run` → `make run` (if target exists)
    - `build` → `make`
- [x] Output aliases as shell-executable text
- [ ] Ensure aliases are local to the directory context (shell handles scope)
- [ ] Option to disable env injection per-directory (`.cfmignore` or config)

---

## Phase 5: File Operations (`fm mv`) — STUB (`fm mv`)

> When moving files, print a temporary split-context dashboard for visual confirmation.

- [ ] `fm mv <sources...> <dest>`:
  - [ ] Accept multiple source files/globs and a destination
  - [ ] Before executing, print a **Split Context / Double Pane Dashboard**:
    ```text
    ┌─ 📤 SOURCE: ~/projects/backend ───────┐  ┌─ 📥 DESTINATION: ~/projects/backend/src ─┐
    │  📄 main.rs         (450 B)           │  │  📂 utils/                                │
    │  📄 server.rs       (1.2 KB)          │  │  📄 lib.rs                                │
    │                                       │  │  ⚠️  Collision: server.rs already exists! │
    └───────────────────────────────────────┘  └───────────────────────────────────────────┘
    ```
  - [ ] Detect name collisions before acting
  - [ ] Offer collision resolution: `--overwrite`, `--rename`, `--skip`, or interactive prompt (standard stdin read, not raw mode)
  - [ ] Print confirmation table after move completes
  - [ ] Support `--dry-run`
  - [ ] Accept piped input: `fd *.rs | fm mv src/`

---

## Phase 6: Ephemeral Clipboard — STUB (`fm yank` / `fm paste`)

> Cross-terminal, cross-session file clipboard. Stateless binary, persistent dotfile.

- [ ] `fm yank <paths...>`:
  - [ ] Accept glob patterns and multiple paths
  - [ ] Resolve to absolute paths
  - [ ] Validate paths exist
  - [ ] Write to `~/.local/share/cfm/clipboard.json`
  - [ ] Print confirmation: "📋 Yanked N files."
  - [ ] Show yanked items in a mini table
- [ ] `fm paste`:
  - [ ] Read from `~/.local/share/cfm/clipboard.json`
  - [ ] Copy (not move by default?) — DECIDE: default to copy or move?
  - [ ] Handle name collisions (prompt or auto-rename)
  - [ ] Show progress bar for large operations (`indicatif`)
  - [ ] Print confirmation with file list
  - [ ] Option: `fm paste --move` to move instead of copy
- [ ] `fm clipboard` / `fm cb`:
  - [ ] Show current clipboard contents
  - [ ] Allow clearing: `fm cb --clear`
- [ ] Support relative path resolution from yank location
- [ ] Handle symlinks appropriately

---

## Phase 7: Safe File Operations — STUB (`fm rm` / `fm trash` / `fm open`)

> Visual, safe mutators with confirmation and undo awareness.

- [ ] `fm rm <paths...>`:
  - [ ] Print files to be deleted in a table (with sizes, types)
  - [ ] Require confirmation for >1 file or directories
  - [ ] `--force` / `-f` flag to skip confirmation
  - [ ] `--dry-run` flag to preview without deleting
  - [ ] Rich output: "🗑️  Deleted: file.txt (1.2 KB)"
  - [ ] **Accept piped input:** `fd -e log --changed-before 7d | fm rm`
- [ ] `fm trash <paths...>`:
  - [ ] Move to system trash (`trash` crate for cross-platform)
  - [ ] Same confirmation flow as `rm`
  - [ ] Print "🗑️  Trashed: file.txt"
  - [ ] **Accept piped input:** `fd *.tmp | fm trash`
- [ ] `fm open [paths...]`:
  - [ ] Open files with default application based on extension
  - [ ] Map extensions: `.png` → `xdg-open`/`open`, `.pdf` → viewer, `.md` → `$EDITOR`
  - [ ] **Accept piped input:** `rg "database_url" -l | fm open`
  - [ ] Print one-line confirmation: "🖥️  Opened: config.toml"
  - [ ] `--dry-run` to preview what would open
- [ ] `fm undo` (optional v2):
  - [ ] Log operations to `~/.local/share/cfm/undo.log`
  - [ ] Allow undoing last operation
- [ ] Handle read-only files and permission errors gracefully

---

## Phase 8: Smart Piping — STUB & Actions (`fm do`)

> The ultimate pipe destination. Receive file paths, take smart action.

- [ ] `fm do`:
  - [ ] Read file paths from stdin (piped)
  - [ ] Detect file types from extensions
  - [ ] Map to default actions:
    - `.rs`, `.py`, `.js`, `.ts`, `.toml`, `.json` → `$EDITOR`
    - `.sh` → `bash` (with confirmation)
    - `.png`, `.jpg`, `.svg` → `xdg-open` / `open`
    - `.md` → `$EDITOR` or `glow`
    - `.pdf` → `xdg-open`
    - directories → `cd` (shell wrapper needed)
  - [ ] Print one-line confirmation before acting
  - [ ] Support `--dry-run`
  - [ ] Support `--action <name>` to override (e.g., `fm do --action cat`)
- [ ] `fm peek <file>` (optional):
  - [ ] Syntax-highlighted file preview (`syntect`)
  - [ ] Limit to first N lines (default 50)
  - [ ] Show file metadata header

---

## Phase 9: Directory Stats — STUB (`fm stats`)

> Deep synthesis chart for the current directory.

- [ ] `fm stats [path]`:
  - [ ] Total files, directories, symlinks
  - [ ] Total size + largest files
  - [ ] File type breakdown (pie chart or bar chart via ASCII/Unicode)
  - [ ] Depth distribution
  - [ ] Modification time histogram (today, this week, this month, older)
  - [ ] Git stats if applicable (commits, contributors)
  - [ ] Top 10 largest files table
  - [ ] Top 10 oldest files table
  - [ ] `--json` output for scripting

---

## Phase 10: Spatial Memory — STUB (`fm pin` / `fm jump` / `fm root`)

> Bookmark directories and jump instantly.

- [ ] `fm pin <name>`:
  - [ ] Save current directory to `~/.local/share/cfm/pins.json`
  - [ ] Validate name (no spaces, unique)
  - [ ] Print: "📌 Pinned: name -> /absolute/path"
- [ ] `fm jump <name>`:
  - [ ] Look up pin and print `cd` command for shell wrapper
  - [ ] Hidden/internal: `fm jump --print-cd <name>` outputs `cd /path`
  - [ ] Shell wrapper catches output and executes `cd`
  - [ ] Support partial name matching (fuzzy)
  - [ ] After jumping, banner auto-prints via `chpwd` hook
- [ ] `fm pins`:
  - [ ] List all pins in a table
  - [ ] Show name, path, last accessed
- [ ] `fm unpin <name>`:
  - [ ] Remove a pin
- [ ] `fm root`:
  - [ ] Jump to the root of the current Git repository
  - [ ] Hidden/internal: `fm root --print-cd` outputs `cd /absolute/repo/root`
  - [ ] Shell wrapper catches output and executes `cd`
  - [ ] Print: "⬆️  Jumped to repo root: /path/to/repo"
- [ ] Shell aliases for micro-jumps (documented in hook setup):
  - [ ] `..` → `cd ..` (standard shell alias, banner auto-triggers)
  - [ ] `-` → `cd -` (standard shell alias, banner auto-triggers)
  - [ ] `~` → `cd ~` (standard shell alias, banner auto-triggers)
- [ ] Frecency sorting for pins (optional v2)

### Shell Integration for Jump:
```bash
# Shell wrapper function
fm() {
  if [[ "$1" == "jump" ]]; then
    local target=$(command fm jump --print-cd "$2")
    if [[ -n "$target" ]]; then
      eval "$target"
      command fm banner "$PWD"
    fi
  else
    command fm "$@"
  fi
}
```

---

## Phase 11: Session Management — STUB (`fm save-session` / `fm load-session`)

> Save and restore workspace contexts (terminal tabs, working directories).

- [ ] `fm save-session <name>`:
  - [ ] Save current directory to session profile
  - [ ] Optionally save environment variables (whitelist-based)
  - [ ] Print: "💾 Session saved: my-project"
- [ ] `fm load-session <name>`:
  - [ ] Restore directory and context
  - [ ] Shell wrapper executes `cd` to restored path
  - [ ] Print banner of restored directory
- [ ] `fm sessions`:
  - [ ] List all saved sessions
  - [ ] Show name, path, last accessed
- [ ] `fm delete-session <name>`:
  - [ ] Remove a session profile
- [ ] Store sessions in `~/.local/share/cfm/sessions.json`

---

## Phase 12: Directory Comparison (`fm diff`)

> Compare two directories visually.

- [ ] `fm diff <dir1> <dir2>`:
  - [ ] Show added, removed, modified, identical files
  - [ ] Side-by-side table or unified view
  - [ ] File size differences
  - [ ] `--json` output
  - [ ] `--shallow` (top-level only) vs `--deep` (recursive)

---

## Phase 13: Shell Completions & Autocomplete

> God-tier autocomplete without interactive prompts.

- [ ] Generate shell completions via `clap_complete`:
  - [ ] Bash
  - [ ] Zsh
  - [ ] Fish
  - [ ] PowerShell (optional)
- [ ] `fm complete --dir <path>`:
  - [ ] Output directory-specific completions (files, subdirs)
  - [ ] Format for shell consumption
- [ ] Document installation of completions
- [ ] Dynamic completions: `fm yank <TAB>` completes files

---

## Phase 14: Configuration

- [ ] `fm config` command or `~/.config/cfm/config.toml`:
  - [ ] Banner appearance (icons on/off, colors theme, compact vs expanded)
  - [ ] Default behavior flags
  - [ ] Mutator command list for smart refresh
  - [ ] Project type detection rules
  - [ ] Custom alias templates per project type
  - [ ] Disable banner in specific directories (regex/paths)
- [ ] `fm config --edit` opens config in `$EDITOR`
- [ ] Sensible defaults (zero-config philosophy)

---

## Phase 15: Polish & Edge Cases

- [ ] Handle terminals with no Unicode/font support (fallback to ASCII)
- [ ] Handle terminals with no color support (`NO_COLOR` env, `TERM=dumb`)
  - [ ] Respect `NO_COLOR` standard
  - [ ] Respect `CLICOLOR` / `CLICOLOR_FORCE`
- [ ] Handle very long paths gracefully (truncate middle)
- [ ] Handle permission-denied directories gracefully
- [ ] Handle broken symlinks
- [ ] Handle circular symlinks
- [ ] Graceful degradation when `git` repo is corrupted
- [ ] Windows support (optional v2 — use `directories`, `trash`, `normpath`)
- [ ] macOS support (ensure `open` command, trash works)
- [ ] Linux support (primary target)

---

## Phase 16: Testing

- [ ] Unit tests for:
  - [ ] Project type detection
  - [ ] Path resolution and normalization
  - [ ] Clipboard serialization/deserialization
  - [ ] Pin storage and retrieval
  - [ ] Git status parsing
  - [ ] TTY detection logic
- [ ] Integration tests:
  - [ ] CLI argument parsing (all subcommands)
  - [ ] Banner output in TTY vs pipe
  - [ ] JSON output validation
  - [ ] End-to-end: `fm yank` → `fm paste` → verify
- [ ] Performance benchmarks:
  - [ ] Banner latency in empty dir (<5ms)
  - [ ] Banner latency in 1k file dir (<20ms)
  - [ ] Banner latency in 10k file dir (<50ms)
- [ ] Test fixtures:
  - [ ] Mock Git repo
  - [ ] Mock Node project
  - [ ] Mock Rust project
  - [ ] Mock Python project

---

## Phase 17: Documentation

- [ ] `README.md`:
  - [ ] The manifesto (zero-hostage philosophy)
  - [ ] Installation (`cargo install cfm`)
  - [ ] Shell hook setup (`fm install-hook`)
  - [ ] Quick start / demo GIF
  - [ ] Command reference
  - [ ] Configuration guide
  - [ ] Comparison with other tools (ranger, yazi, nnn, lsd, eza)
  - [ ] Troubleshooting
- [ ] Man page (`fm.1`) or generated help
- [ ] Shell completion docs
- [ ] Contributing guide
- [ ] Changelog
- [ ] crates.io metadata

---

## Phase 18: Packaging & Distribution

- [ ] Publish to crates.io as `cfm` (binary `fm`)
- [ ] GitHub Actions CI:
  - [ ] Run tests on Linux
  - [ ] Run clippy + fmt checks
  - [ ] Build release binaries (Linux x86_64, aarch64)
  - [ ] Automated crates.io publish on tag
- [ ] Create GitHub releases with prebuilt binaries
- [ ] Homebrew formula (optional)
- [ ] AUR package (optional)
- [ ] Nix flake (optional)

---

## Open Questions / Decisions to Make

1. [x] **Yank default behavior:** Copy (not move). Add `--move` flag for move.
2. [x] **Banner auto-refresh:** Only on `chpwd` + mutators after heavy commands. Never continuous.
3. [x] **`fm` with no args:** Prints the banner (equivalent to `fm banner`). This is the "Type `fm`. Get context." behavior.
4. [x] **`fm ls` vs `fm banner`:** `fm ls` is NOT a separate command. The banner IS the enhanced directory listing. For full raw listing, use `ls` or `fm banner --raw`.
5. [ ] `fm do` action map: Make it configurable? (`~/.config/cfm/actions.toml`)
6. [x] **Watch mode / live updates:** Strictly NO. Documented in manifesto: breaks terminal contract, causes cursor desync.
7. [ ] Should `fm rm` have a recycle-bin style undo, or just rely on `fm trash`?
8. [ ] Cross-platform trash: `trash` crate handles it, but verify Linux dependencies
9. [x] **`fm find`:** NO. Defer to `fd` and `rg`. `cfm` integrates with them via piping, doesn't replace them.
10. [x] **Multi-select yank:** Yes, `fm yank *.log src/*.rs` is supported via glob expansion.
11. [ ] **State file format:** Unified `state.json` vs separate files (clipboard.json, pins.json, sessions.json)?
12. [ ] **Shell wrapper scope:** Should the wrapper intercept `fm up`, `fm back`, `fm root`, or just `fm jump`?
13. [ ] **`fm open` vs `fm do`:** Are these the same command? `fm open` could be an alias for `fm do --action open`.
14. [ ] **Double Pane for other operations:** Should `fm cp` also show split context, or only `fm mv`?

---

## Design Principles (Checklist for Every Feature)

Before adding ANY new feature, verify:

- [ ] Does it respect the "print once and die" rule?
- [ ] Does it avoid TUI / alternate screen buffer?
- [ ] Does it avoid background daemons / watchers?
- [ ] Does it work without a mouse?
- [ ] Does it support `--raw` / `--json` for piping?
- [ ] Does it complete in <50ms for typical directories?
- [ ] Does it degrade gracefully on narrow terminals?
- [ ] Does it respect `NO_COLOR`?
- [ ] Does it add value beyond what `fd | fzf | xargs` can do?
- [ ] Does it maintain the "Invisible File Manager" / "Passive File Manager" principle?
- [ ] Is it keyboard-centric (home row, minimal keystrokes)?

---

## Done Definition

A phase is done when:
1. Code is written, compiles, and passes `cargo test`
2. Feature works in both TTY (rich) and pipe (raw/JSON) modes
3. Help text is complete (`--help` for all subcommands)
4. No clippy warnings
5. Documented in README or inline docs

---

*Last updated: 2026-05-24*
