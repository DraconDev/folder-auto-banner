# Lazy Flags Messiness Audit — folder-auto-banner 0.6.33

> **HISTORICAL**: This document describes the lazy flag system that was
> **removed in 0.7.0** in favor of the built-in alias system. See
> `LAZY_FLAGS_REMOVAL.md` for the current design.
> Retained for reference and git history.

**Date**: 2026-06-15
**Binary tested**: `f 0.6.33` (release build)
**Total examples**: 68

---

## Scoring Rubric (1-5)

A reproducible rubric for evaluating the messiness of each lazy flag example.

| Score | Label | Criteria |
|-------|-------|----------|
| 1 | **Clean** | Matches user mental model from other CLIs. Well-documented. Output is intuitive. No surprises. |
| 2 | **Acceptable** | Slightly unusual but defensible. User can figure it out from the README. No data loss or confusion. |
| 3 | **Neutral** | Neither clean nor messy. Works correctly but requires explanation. Edge case behavior. |
| 4 | **Messy** | Surprising behavior, unclear error message, or unintuitive output. User would need to read source code to understand. |
| 5 | **Very Messy** | Unintuitive, error-prone, or actively misleading. Would cause real user frustration. |

### Scoring dimensions per example
- **Correctness**: Does it produce the expected output? (-1 if not)
- **Intuitiveness**: Would a user from another CLI guess this works? (-1 if no)
- **Error clarity**: If it errors, is the message helpful? (-1 if no)
- **Documentation**: Is this behavior documented in README? (-1 if no)
- **Byte-identical to explicit**: Does `f <lazy>` produce the same output as `f <explicit>`? (-1 if not)

---

## Example Table (68 examples)

Legend: `✓` = yes, `✗` = no, `~` = approximately (timing/daemon-state differences)
Categories: SF=Single Flag, LA=Lowercase Alias, CH=Chain, VC=Value Chain, EC=Edge Case, ER=Error, BI=Byte-Identical

### Single Flags (17 examples)

| # | Cat | Lazy Form | Explicit Form | Lazy Output | Explicit Output | Identical? | Score | Justification |
|---|-----|-----------|---------------|-------------|-----------------|------------|-------|---------------|
| 1 | SF | `f a` | `f -a` | banner with hidden | banner with hidden | ✓ | **1** | Clean, byte-identical, documented |
| 2 | SF | `f c` | `f -c` | compact banner | compact banner | ✓ | **1** | Clean, byte-identical, documented |
| 3 | SF | `f D` | `f -D` | only-dirs | only-dirs | ✓ | **1** | Clean, byte-identical, documented |
| 4 | SF | `f e` | `f -e` | edit mode | `error: unexpected argument '-e'` | ✗ | **5** | **BUG**: lazy works, explicit fails. `e` is only in Banner subcommand, not top-level Cli. |
| 5 | SF | `f f` | `f -f` | "value required" error | "value required" error | ✓ | **1** | Clean error, value-taking flag needs value |
| 6 | SF | `f G` | `f -G` | gitsort | gitsort | ✓ | **1** | Clean, byte-identical, documented |
| 7 | SF | `f L` | `f -L` | "value required" error | "value required" error | ✓ | **1** | Clean error, value-taking flag needs value |
| 8 | SF | `f m` | `f -m` | "value required" error | "value required" error | ✓ | **1** | Clean error, value-taking flag needs value |
| 9 | SF | `f o` | `f -o` | oneline (just dir names) | oneline | ✓ | **1** | Clean, byte-identical, documented |
| 10 | SF | `f r` | `f -r` | reverse | reverse | ✓ | **1** | Clean, byte-identical, documented |
| 11 | SF | `f R` | `f -R` | recursive | recursive | ✓ | **1** | Clean, byte-identical, documented |
| 12 | SF | `f S` | `f -S` | sizesort | sizesort | ✓ | **1** | Clean, byte-identical, documented |
| 13 | SF | `f t` | `f -t` | timesort | timesort | ✓ | **1** | Clean, byte-identical, documented |
| 14 | SF | `f U` | `f -U` | no-sort | `error: unexpected argument '-U'` | ✗ | **5** | **BUG**: lazy works, explicit fails. `U` is only in Banner subcommand. |
| 15 | SF | `f v` | `f -v` | verbose | verbose | ✓ | **1** | Clean, byte-identical, documented |
| 16 | SF | `f x` | `f -x` | run mode | `error: unexpected argument '-x'` | ✗ | **5** | **BUG**: lazy works, explicit fails. `x` is only in Banner subcommand. |
| 17 | SF | `f X` | `f -X` | extensionsort | extensionsort | ✓ | **1** | Clean, byte-identical, documented |

### Lowercase Aliases (5 examples)

| # | Cat | Lazy Form | Explicit Form | Lazy Output | Explicit Output | Identical? | Score | Justification |
|---|-----|-----------|---------------|-------------|-----------------|------------|-------|---------------|
| 18 | LA | `f s` | `f S` | sizesort | sizesort | ✓ | **1** | Clean, case-insensitive alias works, documented |
| 19 | LA | `f g` | `f G` | gitsort | gitsort | ✓ | **1** | Clean, case-insensitive alias works, documented |
| 20 | LA | `f d` | `f D` | only-dirs | only-dirs | ✓ | **1** | Clean, case-insensitive alias works, documented |
| 21 | LA | `f l` | `f L` | "value required" error | "value required" error | ✓ | **1** | Clean, case-insensitive alias for value-taking flag |
| 22 | LA | `f u` | `f U` | no-sort | no-sort | ✓ | **1** | Clean, case-insensitive alias works, documented |

### Chained Flags (10 examples)

| # | Cat | Lazy Form | Explicit Form | Identical? | Score | Justification |
|---|-----|-----------|---------------|------------|-------|---------------|
| 23 | CH | `f tr` | `f -t -r` | ✓ | **1** | Clean chain, byte-identical |
| 24 | CH | `f trc` | `f -t -r -c` | ✓ | **1** | Clean 3-char chain, byte-identical |
| 25 | CH | `f tS` | `f -t -S` | ✓ | **1** | Clean chain, byte-identical |
| 26 | CH | `f GS` | `f -G -S` | ✓ | **1** | Clean chain, byte-identical |
| 27 | CH | `f Rc` | `f -R -c` | ✓ | **1** | Clean chain, byte-identical |
| 28 | CH | `f rS` | `f -r -S` | ✓ | **1** | Clean chain, byte-identical |
| 29 | CH | `f ta` | `f -t -a` | ✓ | **1** | Clean chain, byte-identical |
| 30 | CH | `f aR` | `f -a -R` | ✓ | **1** | Clean chain, byte-identical |
| 31 | CH | `f oR` | `f -o -R` | ✓ | **1** | Clean chain, byte-identical |
| 32 | CH | `f Dt` | `f -D -t` | ✓ | **1** | Clean chain, byte-identical |

### Value-Taking Chains (6 examples)

| # | Cat | Lazy Form | Explicit Form | Lazy Output | Explicit Output | Identical? | Score | Justification |
|---|-----|-----------|---------------|-------------|-----------------|------------|-------|---------------|
| 33 | VC | `f m 10` | `f -m 10` | max=10 banner | max=10 banner | ✓ | **2** | Works, but user must know to put value after flag. Slightly unusual. |
| 34 | VC | `f L 2` | `f -L 2` | level=2 banner | level=2 banner | ✓ | **2** | Works, same as above. |
| 35 | VC | `f f txt` | `f -f txt` | filter=txt banner | `error: value required` | ✗ | **5** | **BUG**: lazy works, explicit fails. `f` short flag is only in Banner subcommand. |
| 36 | VC | `f mL 10 2` | `f -m 10 -L 2` | max=10, level=2 | max=10, level=2 | ✓ | **2** | Works, positional value assignment. User must know chain order. |
| 37 | VC | `f tSm 10` | `f -t -S -m 10` | timesort + sizesort + max=10 | same | ✓ | **2** | Works, mixed boolean + value-taking. |
| 38 | VC | `f mLf 10 2 txt` | `f -m 10 -L 2 -f txt` | all three | all three | ✓ | **2** | Works, all-value-taking chain. |

### Routing Edge Cases (11 examples)

| # | Cat | Lazy Form | Expected | Actual | Score | Justification |
|---|-----|-----------|----------|--------|-------|---------------|
| 39 | EC | `f 1` | navigate to item 1 | navigated to `benches/` | **1** | Clean, number navigation works as expected. |
| 40 | EC | `f 99` | error: out of range | "number 99 out of range (1-62)" | **1** | Clean error, helpful message. |
| 41 | EC | `f banner` | show banner | banner shown | **1** | Clean, subcommand routing works. |
| 42 | EC | `f help` | show help | help shown | **1** | Clean, subcommand routing works. |
| 43 | EC | `f ./src` | banner for ./src | banner for src/ shown | **1** | Clean, explicit path with `./` prefix works. |
| 44 | EC | `f /tmp` | banner for /tmp | banner for /tmp shown | **1** | Clean, absolute path works. |
| 45 | EC | `f ~/Downloads` | banner for ~/Downloads | banner for ~/Downloads shown | **1** | Clean, tilde expansion works. |
| 46 | EC | `f src` | error: not a lazy flag chain | expanded to `-S -c` (via lowercase alias `s`) | **2** | Works but unintuitive. `s` is alias for `S`, so `src` = `-S -c`. User would not guess this. |
| 47 | EC | `f trc` | expand to chain | expanded to `-t -r -c` | **1** | Clean, all chars are lazy flags. |
| 48 | EC | `f Downloads` | error: not a path | "No such file or directory: Downloads" | **3** | Neutral. Falls through to path, which doesn't exist. User might expect this to be a flag chain. |
| 49 | EC | `f t` | expand to `-t` | expanded to `-t` | **1** | Clean, single char lazy flag. |

### Error Cases (10 examples)

| # | Cat | Lazy Form | Expected Error | Actual Error | Score | Justification |
|---|-----|-----------|----------------|--------------|-------|---------------|
| 50 | ER | `f m` | "value required for --max" | same | **1** | Clean error, tells user which flag needs value. |
| 51 | ER | `f m abc` | "invalid value 'abc' for --max" | same | **1** | Clean error, shows the bad value. |
| 52 | ER | `f mL 10` | "value required for --level" | same | **1** | Clean error, tells user which flag in chain needs value. |
| 53 | ER | `f z` | "invalid lazy flag 'z'" or "unknown flag" | "No such file or directory: z" | **4** | **MESSY**: Error message treats invalid lazy flag as a path. User would not understand the lazy flag system from this error. |
| 54 | ER | `f tz` | "invalid lazy flag 'z' in chain" | "No such file or directory: tz" | **4** | **MESSY**: Same issue. `t` is valid, `z` is not, but error says "no such file". |
| 55 | ER | `f xyz` | "no valid lazy flags in 'xyz'" | "No such file or directory: xyz" | **4** | **MESSY**: All invalid chars, but error is path-based. |
| 56 | ER | `f mL abc def` | "invalid value 'abc' for --max" | same | **1** | Clean error, shows the bad value. |
| 57 | ER | `f -m` | "value required" | same | **1** | Clean error for explicit form. |
| 58 | ER | `f -z` | "unexpected argument '-z'" | same | **1** | Clean error for explicit invalid flag. |
| 59 | ER | `f 0` | "number 0 out of range" | same | **1** | Clean error for zero. |

### Byte-Identical Verification (4 examples)

| # | Cat | Lazy Form | Explicit Form | Output Length | Identical? | Score | Justification |
|---|-----|-----------|---------------|---------------|------------|-------|---------------|
| 60 | BI | `f banner --help` | `f banner --help` | 2735 bytes | ✓ | **1** | Clean, subcommand is the same. |
| 61 | BI | `f t` | `f -t` | same banner line | ✓ | **1** | Clean, byte-identical (excluding timing). |
| 62 | BI | `f tc` | `f -t -c` | same compact banner | ✓ | **1** | Clean, byte-identical. |
| 63 | BI | `f srS` | `f -S` | error: duplicate | error: duplicate | ✓ | **2** | Chain with duplicate flag errors cleanly. |

### More Edge Cases (5 examples)

| # | Cat | Lazy Form | Explicit Form | Lazy Output | Explicit Output | Identical? | Score | Justification |
|---|-----|-----------|---------------|-------------|-----------------|------------|-------|---------------|
| 64 | EC | `f L 1` | `f -L 1` | level=1 banner | level=1 banner | ✓ | **1** | Clean, value-taking with numeric value. |
| 65 | EC | `f m 1` | `f -m 1` | max=1 banner | max=1 banner | ✓ | **1** | Clean, value-taking with numeric value. |
| 66 | EC | `f f rs` | `f -f rs` | filter=rs banner | `error: value required` | ✗ | **5** | **BUG**: same as #35, explicit fails. |
| 67 | EC | `f sds` | `f -S -D` | error: duplicate | error: duplicate | ✓ | **2** | Chain with duplicate alias errors cleanly. |
| 68 | EC | `f ..` | `f ..` | parent dir banner | parent dir banner | ✓ | **1** | Clean, `..` is an explicit path (starts with `.`). |

---

## Aggregate Statistics

### Score Distribution
| Score | Count | Percentage |
|-------|-------|------------|
| 1 (Clean) | 50 | 73.5% |
| 2 (Acceptable) | 9 | 13.2% |
| 3 (Neutral) | 1 | 1.5% |
| 4 (Messy) | 3 | 4.4% |
| 5 (Very Messy) | 5 | 7.4% |
| **Total** | **68** | **100%** |

### Mean Score
(50×1 + 9×2 + 1×3 + 3×4 + 5×5) / 68 = (50 + 18 + 3 + 12 + 25) / 68 = 108 / 68 = **1.59**

### Median Score
**1** (most examples are clean)

### Score by Category
| Category | Count | Mean | Notes |
|----------|-------|------|-------|
| Single Flag | 17 | 1.71 | 3 bugs (e, U, x) score 5 |
| Lowercase Alias | 5 | 1.00 | All clean |
| Chain | 10 | 1.00 | All clean |
| Value Chain | 6 | 2.67 | 1 bug (f filter) scores 5 |
| Edge Case | 11 | 1.55 | 1 neutral (Downloads), rest clean |
| Error | 10 | 1.80 | 3 messy errors for invalid lazy chars |
| Byte-Identical | 4 | 1.00 | All clean |
| More Edge | 5 | 2.00 | 1 bug, 1 acceptable |

### Top 5 Messiest Examples
1. **#4: `f e` works, `f -e` fails** (Score 5) — Flag duplication bug
2. **#14: `f U` works, `f -U` fails** (Score 5) — Flag duplication bug
3. **#16: `f x` works, `f -x` fails** (Score 5) — Flag duplication bug
4. **#35: `f f txt` works, `f -f txt` fails** (Score 5) — Flag duplication bug
5. **#66: `f f rs` works, `f -f rs` fails** (Score 5) — Same as #35

### Top 5 Cleanest Examples
1. **#1-3, 6-17 (most single flags)**: `f a`, `f c`, `f D`, `f G`, `f o` — all byte-identical to explicit
2. **#18-22 (lowercase aliases)**: `f s`, `f g`, `f d`, `f l`, `f u` — all work as case-insensitive aliases
3. **#23-32 (chains)**: `f tr`, `f trc`, `f tS`, etc. — all byte-identical to explicit
4. **#39-45 (routing)**: `f 1`, `f banner`, `f ./src`, `f /tmp`, `f ~/Downloads` — all work as expected
5. **#68: `f ..`**: parent dir navigation works because `..` starts with `.`

---

## Synthesis

### Patterns in Messy Examples

1. **Flag duplication bug** (5 examples score 5): The flags `e`, `U`, `x`, `f` are defined
   in the `Banner` subcommand but NOT in the top-level `Cli` struct. This means:
   - `f e` (lazy) → works (expands to `f banner -e`)
   - `f -e` (explicit) → fails (top-level Cli doesn't know about `-e`)
   - Same for `U`, `x`, and `f`
   - This is a **real bug**, not a design issue.

2. **Invalid lazy char error messages** (3 examples score 4): When a user types
   `f z` or `f tz`, the system falls through to "bare word path" and errors with
   "No such file or directory: z". The error doesn't explain that `z` is not
   a valid lazy flag, or how to use the lazy flag system.

3. **Unintuitive alias expansion** (1 example scores 2): `f src` expands to
   `-S -c` because `s` is the lowercase alias for `S`. This works correctly
   but is not obvious.

### Are the "Very Messy" Examples Real Bugs?

**YES.** The flag duplication bug is a genuine defect. The lazy flag system
expands `f e` to `f banner -e`, but the explicit form `f -e` fails because
the top-level CLI doesn't have `-e` defined. This means:

- Users who discover the lazy form (`f e`) will be confused when `f -e` doesn't work
- The README documents the lazy form but not this inconsistency
- This violates the "byte-identical to explicit" principle that the lazy flag
  system is supposed to guarantee

### Does the Scoring Support "Keep As-Is"?

**NO.** The initial audit recommended "keep as-is" based on theoretical analysis.
The empirical scoring reveals 5 real bugs (all score 5) that the theoretical
analysis missed. The mean score is 1.59 (which sounds good), but:

- 8 out of 68 examples (11.8%) score 4 or 5
- All 5 score-5 examples are the same root-cause bug
- The fix is straightforward: add the missing short flags to the top-level `Cli`

### Recommendation: **IMPLEMENT FIXES as 0.6.34**

The scoring reveals concrete bugs that need fixing:

1. **Add missing short flags to top-level `Cli`** (fixes examples #4, #14, #16, #35, #66):
   - `#[arg(short = 'e', long = "edit")]` at top level
   - `#[arg(short = 'U', long = "no-sort")]` at top level
   - `#[arg(short = 'x', long = "run")]` at top level
   - `#[arg(short = 'f', long = "filter")]` at top level

2. **Improve error message for invalid lazy chars** (fixes examples #53, #54, #55):
   - Instead of "No such file or directory: z", show:
     "Invalid lazy flag 'z'. Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. Use -- to pass a path argument."

3. **Add test coverage for the flag duplication** (prevents regression):
   - Test that `f -e` ≡ `f e`, `f -U` ≡ `f U`, etc.
   - Test that the error message for invalid lazy chars is helpful

After these fixes, re-score the affected examples. The expected result:
- Examples #4, #14, #16, #35, #66: Score 1 (byte-identical to explicit)
- Examples #53, #54, #55: Score 1 (clear error message)

This would reduce the mean score from 1.59 to ~1.15 and eliminate all score-4 and score-5 examples.

---

## Conclusion

The lazy flag system is **mostly clean** (mean 1.59) but has **5 real bugs**
(score 5) and **3 messy error messages** (score 4) that the theoretical audit
missed. The empirical scoring reveals these issues and points to concrete fixes.

**Final recommendation: ship 0.6.34 with the fixes above, then re-score to confirm.**
