# cfm Audit — Comparison with lsd/exa

## What We Have (Our Edge)

### Unique Features (not in lsd/exa)
- **Build status** — shows if project builds cleanly
- **TODO count** — counts TODOs/FIXMEs
- **Languages breakdown** — shows code composition
- **Port detection** — shows listening ports
- **Docker status** — shows running containers
- **Git context** — last commit, commits today, branches
- **Cached test results** — shows last test run
- **Daemon** — pre-computes data for instant display

### Standard Features (we have)
- File listing with permissions, owner, group, size, date, name
- Icons for file types
- Git status per file
- Sorting (name, size, date, type)
- Tree view
- Hidden files
- JSON/raw output
- TTY detection

---

## What lsd Has (We're Missing)

### Display Modes
| Feature | lsd | cfm | Notes |
|---------|-----|-----|-------|
| `--permission` | rwx, octal, attributes | rwx only | Could add octal mode |
| `--size` | default, short, bytes | default only | Could add short/bytes |
| `--date` | date, locale, relative | date only | Could add relative |
| `--classify` | append */=>@| | no | Useful for quick identification |
| `-1` | one file per line | no | Could add |
| `-R` | recursive | no | Tree view covers this |
| `--header` | show block headers | no | Could add |
| `--hyperlink` | attach hyperlinks | no | Nice for terminals |

### Sorting
| Feature | lsd | cfm | Notes |
|---------|-----|-----|-------|
| `--sort` | size, time, version, extension, git | name, size, date, type, git, extension, version | We have more |
| `-t` | timesort | yes | Same |
| `-S` | sizesort | yes | Same |
| `-X` | extensionsort | yes | Same |
| `-G` | gitsort | yes | Same |
| `-v` | versionsort | yes | Same |
| `-U` | no-sort | yes | Same |
| `--group-dirs` | none, first, last | group-dirs first/last | Same |
| `-r` | reverse | yes | Same |

### Display
| Feature | lsd | cfm | Notes |
|---------|-----|-----|-------|
| `--tree` | yes | yes | Same |
| `--depth` | yes | tree [depth] | Same |
| `--blocks` | customizable columns | no | Could add |
| `--total-size` | show total size | no | Could add |
| `--inode` | show inode | no | Could add |
| `--links` | show hard links | no | Could add |
| `-g` | git status | yes | We have more |

### Permissions
| Feature | lsd | cfm | Notes |
|---------|-----|-----|-------|
| `--permission` | rwx, octal, attributes, disable | rwx only | Could add octal |
| `--truncate-owner` | truncate user/group | no | Could add |
| `--no-symlink` | hide symlink target | no | Could add |

---

## What We Have (They Don't)

### Our Unique Edge
1. **Daemon** — pre-computes data, instant display
2. **Build status** — shows if project builds
3. **TODO count** — counts TODOs/FIXMEs
4. **Languages breakdown** — shows code composition
5. **Port detection** — shows listening ports
6. **Docker status** — shows running containers
7. **Git context** — last commit, commits today, branches
8. **Cached test results** — shows last test run

### What They Have (We Don't)
1. **`--permission` modes** — octal, attributes
2. **`--size` modes** — short, bytes
3. **`--date` modes** — relative, locale
4. **`--classify`** — append indicators
5. **`--blocks`** — customize columns
6. **`--hyperlink`** — attach hyperlinks
7. **`--header`** — show block headers
8. **`--total-size`** — show total size
9. **`--inode`** — show inode
10. **`--links`** — show hard links
11. **`--truncate-owner`** — truncate names
12. **`--no-symlink`** — hide targets

---

## Recommendations

### High Priority (match lsd features)
1. **`--permission` mode** — add octal support
2. **`--size` mode** — add short/bytes options
3. **`--date` mode** — add relative time
4. **`--classify`** — append indicators (*/=>@|)
5. **`--blocks`** — customize which columns to show

### Medium Priority (nice to have)
6. **`--total-size`** — show total directory size
7. **`--truncate-owner`** — truncate long names
8. **`--no-symlink`** — hide symlink targets
9. **`--hyperlink`** — attach hyperlinks

### Low Priority (rarely needed)
10. **`--inode`** — show inode
11. **`--links`** — show hard links
12. **`-1`** — one file per line (tree covers this)

---

## Our Competitive Advantage

**We're not trying to be lsd/exa** — we're adding context they don't have:
- Build status
- TODO count
- Languages breakdown
- Port detection
- Docker status
- Git context (last commit, commits today)
- Cached test results

**Our edge is instant context** — no other tool shows this much information at a glance.

---

## What We Should Do

1. **Add missing display modes** (permission, size, date) — match lsd
2. **Add classify mode** — useful for quick identification
3. **Add blocks customization** — let users choose what to show
4. **Keep our unique features** — don't remove what makes us special
5. **Improve formatting** — make sure everything is readable
