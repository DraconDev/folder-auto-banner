# Banner Improvement TODO

## Current State
The cfm banner now shows:
- Single-line header with path, project type, size, file/dir counts
- Items in columns with name, count/size, type, modified time
- Smart hidden file handling

## Desired Improvements

### 1. More Information in Header
- [x] Git status indicators (dirty, staged, ahead/behind) - ✅ Shows "1 modified", "3 untracked", etc.
- [ ] Last modified date of directory itself
- [ ] Free/used space for filesystem
- [ ] Parent directory size

### 2. Better Column Layout
- [ ] Responsive columns based on terminal width
- [ ] Optimal column widths (name, size, type, modified, perms?)
- [x] Row-based output (one item per line, clear and readable) - ✅ Done
- [ ] Sort by: name, size, modified, type (default: name)
- [x] Max columns capped at 5 to keep rows manageable - ✅ Done
- [ ] Column-based output (multiple items per row) - deprecated per user preference

### 3. More Item Information
- [ ] Show permissions (rwxr-xr-x)
- [ ] Show owner/group for dirs
- [ ] Show symlink target
- [ ] Show file type icon beyond dir/file
- [ ] Color coding by type/category
- [ ] Size bar visualization (like disk usage bars)

### 4. Context Menu / Quick Actions
- [ ] Show common actions (mv, cp, open)
- [ ] Keyboard shortcuts
- [ ] Preview on hover (hard in CLI, maybe skip)

### 5. Filtering & Search
- [ ] `--hidden` flag to always show hidden
- [ ] `--sort` flag (name, size, modified)
- [ ] `--filter` flag (type, size range, name pattern)
- [ ] `--max` flag (max items per column)

### 6. Performance
- [ ] Cache directory scan results
- [ ] Parallel scanning
- [ ] `--depth` flag for recursive view

### 7. User Preferences
- [ ] `fm config` to save preferences
- [ ] Remember last sort preference
- [ ] Configurable columns
- [ ] Theme colors

### 8. Git Integration
- [x] Show git status icons per item (modified, staged, untracked) - ✅ In header
- [ ] Show which items are in .gitignore
- [ ] Show conflicts in merge
- [ ] Branch comparison (main vs feature)

### 9. Smart Categorization
- [ ] Group by type (Documents, Images, Code, etc.)
- [ ] Show category counts
- [ ] Collapsible groups

### 10. Rich Output Options
- [ ] `--tree` view for subdirectories
- [ ] `--preview` for file contents
- [ ] `--duplicates` find duplicate files
- [ ] `--large` highlight large files

## Priority

### High
1. More informative header (git status, hidden count)
2. Better column sizing
3. Filtering options

### Medium
4. Permissions display
5. Sort options
6. Size visualization

### Low
7. Config system
8. Caching
9. Rich preview modes

## Implementation Notes

### Column Width Strategy
- Read terminal width from `console` crate
- Calculate optimal column count: `term_width / (name_width + size_width + type_width)`
- Minimum column width: 40 chars
- Allow user config for preferred widths

### Git Status Icons
```rust
" "  // clean
"*"  // modified
"+"  // staged
"?"  // untracked
"!"  // conflicts
"~"  // behind
"^"  // ahead
```

### Size Bar Visualization
```rust
// Visual representation of size
fn size_bar(bytes: u64, max: u64) -> String {
    let ratio = (bytes as f64 / max as f64).min(1.0);
    let bars = (ratio * 10.0).round() as usize;
    format!("{:━<10}", "█".repeat(bars))
}
```

## Testing
- [ ] Test with 1000+ items in directory
- [ ] Test with very long filenames
- [ ] Test with unicode filenames
- [ ] Test with symlinks
- [ ] Test with Windows paths
- [ ] Performance benchmark (< 50ms for 10k files)
