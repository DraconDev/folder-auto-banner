# Smart Tree Feature - Experimental Findings

## Date: 2026-06-04

## Objective
Experiment with two approaches for showing subfolder information in the banner:
1. Inline subfolder previews
2. Right-side mini tree display

## Approach 1: Inline Subfolder Preview

### Implementation
- Config option: `inline_preview = true` in `~/.config/fab/config.toml`
- Shows top 2-3 items from subdirectories after a `│` separator
- Only displays when terminal width > current row length + 20 chars

### Example Output
```
📁 src │ lib.rs main.rs mod.rs
📁 tests │ test_banner.rs
```

### Pros
- Simple to implement
- Shows actual file names inside directories
- Compact - doesn't take extra vertical space
- Works well on medium-width terminals (100-120 chars)

### Cons
- Limited to 2-3 items per directory
- Can get truncated on narrow terminals
- Doesn't show directory structure/hierarchy

## Approach 2: Right-Side Mini Tree

### Implementation
- Config option: `mini_tree = true` in `~/.config/fab/config.toml`
- Shows compact tree of top 5 subdirectories on the right side
- Only displays when terminal width > 120 chars
- Uses 1/3 of terminal width (max 40 chars)

### Example Output
```
drwxr-xr-x ... 📁 src
drwxr-xr-x ... 📁 tests              Dev
-rw-r--r-- ... 📄 Cargo.toml          ├── ai-auto-repo-rot-scanner-todo-agent
                                       ├── ai-auto-writer
                                       ├── dracon-platform
                                       ├── src
                                       └── tests
```

### Pros
- Shows directory hierarchy
- Visual tree structure is intuitive
- Doesn't interfere with file listing

### Cons
- Requires wide terminal (120+ chars)
- Only shows directory names, not contents
- Takes extra vertical space
- More complex implementation

## Recommendation

**Use inline preview** for most cases:
- Better for medium-width terminals
- Shows actual file contents
- More information density

**Use mini tree** for wide terminals:
- Better for seeing directory structure
- More visual appeal
- Good for project overview

## Configuration

Both features can be enabled in `~/.config/fab/config.toml`:

```toml
inline_preview = true  # Show subfolder contents inline
mini_tree = true       # Show tree on right side (needs 120+ chars)
```

## Testing

Tested in:
- `~/Dev` (23 directories) - both features work
- `~/Dev/dracon-platform` (12 files) - inline preview shows subfolder contents
- `~/Dev/folder-auto-banner` - standard project view

## Smart Truncation for Big Folders

### Implementation
- Config option: `smart_truncation = true` (enabled by default)
- Detects when folder has more items than `max_display_items` (default: 8)
- Sorts items by git status first, then by recency
- Shows summary of hidden items

### Example Output
```
~/Dev Generic │ 💾 92k │ 📄 0 files │ 📂 23 dirs
...
  📁 avid
  📁 browser-extensions-shared
  📁 dracon-ai-lib
  📁 dracon-platform
  📁 dracon-terminal-engine
  📁 folder-auto-banner
  📁 one-mil-girls
  📁 rust-ai-web-auto
  16 dirs hidden (sorted by git status & recency)
```

### How It Works
1. Counts total items before truncation
2. Sorts by: git changes > recency > name
3. Shows top N items (default: 8)
4. Displays summary of hidden items

## Future Improvements

1. Auto-enable based on terminal width
2. Configurable depth for mini tree
3. Show file counts in mini tree
4. Combine both approaches intelligently
5. Configurable smart_truncation threshold
