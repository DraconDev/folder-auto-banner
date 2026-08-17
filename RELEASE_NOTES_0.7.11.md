# Release Notes — v0.7.11

## Highlights

### 1. Recency-First Flow & Folders-at-Bottom
* **Natural Terminal Eye-Flow:** In terminal listings, your eyes and cursor sit at the **bottom** of the screen. Files are now sorted with recent items at the bottom so today's edits (`README.md`, `Cargo.toml`, etc.) are right next to your prompt.
* **Folders Grouped at Bottom (`group_dirs = "last"`):** Subdirectories (`📁 src`, `📁 tests`, `📁 target`, etc.) are grouped at the very bottom right above your shell prompt, keeping directory jump numbers (`f 23`) instantly accessible without scrolling.

### 2. Smart Pattern Folding for Repetitive Files
* Automatically detects repetitive file clusters (e.g. 40+ `RELEASE_NOTES_*.md`, `001_migration.sql`, `*.log` files).
* Retains the newest 2 entries and folds older archival files into a compact summary, reducing directory clutter from 70+ scrolling lines to **under 25 lines** on a single screen.

### 3. Expressive Git Status Badges
* Replaced generic status dots with standard Git glyphs:
  * `~` (Yellow) for **modified**
  * `+` (Green) for **staged / added**
  * `-` (Red) for **deleted**
  * `?` (Cyan) for **untracked / new**
  * `!` (Red) for **merge conflict**
  * `●` (Dim) for **clean tracked**

### 4. Git Commit Churn & Hotspot Activity
* Fast background parallel querying (`git log --name-only -n 100`) extracts per-file commit frequency.
* High-churn files factor into `git` sorting mode to highlight active codebase hotspots.

### 5. Modern Header & Theme Refinements
* Replaced vertical `│` separators in the banner header with clean, dimmed middle-dot separators (` · `).
* Added `zebra_rows = false` configuration option for clean, uniform dark backgrounds.

### 6. Robustness & Safety Hardening
* Added bounds checks on MP4 duration header parsing to prevent panics on truncated files.
* Bounded SQLite database header reads (64 KiB) to prevent memory allocation spikes.
* Added fast ancestor `.git` checks before spawning Git subprocesses.
