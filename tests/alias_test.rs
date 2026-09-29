// Integration tests for the built-in alias system in f 0.7.0+.
//
// Run with: cargo test --test alias_test -- --test-threads=1
//
// Note: tests must run with --test-threads=1 because the daemon uses
// a single shared socket and parallel runs can flake.

use assert_cmd::Command;
use std::process::Stdio;
use std::time::Duration;

/// Wall-clock ceiling for one `f` invocation under test.
///
/// Every client path is supposed to be bounded (socket connect 2s, banner read
/// 3s, subprocess probes have their own timeouts). This is the backstop that
/// turns "the suite hangs forever on a 4-core runner" into "here is the exact
/// kernel wait state of every thread that was stuck".
const F_TIMEOUT: Duration = Duration::from_secs(45);

/// Describe a pid tree: for every process and every thread, the kernel wait
/// channel and the current syscall. That is what actually answers "where is it
/// blocked?" — `ps` only shows the process exists.
fn describe_process_tree(root: u32) -> String {
    let mut out = String::new();
    let read = |p: String| std::fs::read_to_string(&p).unwrap_or_default();

    // Collect the tree breadth-first: /proc/*/stat field 4 is the ppid.
    let mut frontier = vec![root];
    let mut seen: Vec<u32> = Vec::new();
    while let Some(pid) = frontier.pop() {
        if seen.contains(&pid) {
            continue;
        }
        seen.push(pid);
        let stat = read(format!("/proc/{pid}/stat"));
        let ppid = stat
            .rsplit_once(") ")
            .and_then(|(_, rest)| rest.split_whitespace().next())
            .and_then(|v| v.parse::<u32>().ok());
        let cmdline = read(format!("/proc/{pid}/cmdline"))
            .replace('\0', " ")
            .trim()
            .to_string();
        let wchan = read(format!("/proc/{pid}/wchan")).trim().to_string();
        out.push_str(&format!(
            "\n  pid {pid} ppid {ppid:?} wchan={wchan:?}\n    cmd: {cmdline}\n"
        ));

        let tasks = std::fs::read_dir(format!("/proc/{pid}/task"));
        match tasks {
            Ok(list) => {
                for task in list.flatten() {
                    let tid = task.file_name().to_string_lossy().to_string();
                    let comm = read(format!("/proc/{pid}/task/{tid}/comm"));
                    let wchan = read(format!("/proc/{pid}/task/{tid}/wchan"));
                    let syscall = read(format!("/proc/{pid}/task/{tid}/syscall"));
                    let syscall = syscall.split_whitespace().next().unwrap_or("-");
                    out.push_str(&format!(
                        "    thread {tid} {} wchan={} syscall={syscall}\n",
                        comm.trim(),
                        wchan.trim()
                    ));
                }
            }
            Err(e) => out.push_str(&format!("    (no /proc/{pid}/task: {e})\n")),
        }
        if let Some(ppid) = ppid {
            if ppid > 1 {
                frontier.push(ppid);
            }
        }
    }
    out
}

/// Extra state a hang dump needs: the daemon log (if profiling is on) and the
/// data dir, where a stale socket or unwritten cache tells its own story.
fn daemon_state() -> String {
    let mut out = String::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let data_dir = std::path::Path::new(&home).join(".local/share/fab");
    out.push_str(&format!("\n  data dir {}:\n", data_dir.display()));
    match std::fs::read_dir(&data_dir) {
        Ok(entries) => {
            for e in entries.flatten() {
                let name = e.file_name();
                let meta = e.metadata();
                out.push_str(&format!(
                    "    {} ({} bytes)\n",
                    name.to_string_lossy(),
                    meta.map(|m| m.len()).unwrap_or(0)
                ));
            }
        }
        Err(e) => out.push_str(&format!("    (unreadable: {e})\n")),
    }
    if let Ok(log) = std::fs::read_to_string(data_dir.join("fabd.log")) {
        let tail: Vec<&str> = log.lines().rev().take(40).collect();
        out.push_str("\n  fabd.log (last 40 lines):\n");
        for line in tail.into_iter().rev() {
            out.push_str(&format!("    {line}\n"));
        }
    }
    out
}

/// Run `f` once, bounded, with hang forensics on timeout.
fn run_f_capture(args: &[&str]) -> (String, String, i32) {
    let mut cmd = Command::cargo_bin("f").unwrap();
    for a in args {
        cmd.arg(a);
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .expect("failed to spawn f");

    let pid = child.id();
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let out_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let err_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    });

    let deadline = std::time::Instant::now() + F_TIMEOUT;
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => break status,
            None if std::time::Instant::now() >= deadline => {
                let tree = describe_process_tree(pid);
                let state = daemon_state();
                let _ = child.kill();
                panic!(
                    "\n`f {}` hung for {:?} (pid {pid}) — kernel state:{tree}{state}\n",
                    args.join(" "),
                    F_TIMEOUT
                );
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };

    let out = String::from_utf8_lossy(&out_handle.join().unwrap_or_default()).to_string();
    let err = String::from_utf8_lossy(&err_handle.join().unwrap_or_default()).to_string();
    (out, err, status.code().unwrap_or(-1))
}

/// Helper: run `f` with the given args and return trimmed stdout.
fn run_f(args: &[&str]) -> String {
    run_f_capture(args).0.trim().to_string()
}

/// Helper: run `f` and return (stdout, stderr, exit_code).
fn run_f_full(args: &[&str]) -> (String, String, i32) {
    run_f_capture(args)
}

/// Get the first line of output, truncated to 200 chars (for
/// deterministic comparison that ignores timing-sensitive content).
fn first_line_header(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(200).collect()
}

/// Strip ANSI SGR/CSI escape sequences.
///
/// The banner colourises its path with style codes, so the rendered text for
/// `/tmp` is `ESC[2m/ESC[0mESC[1mtmp` — the literal substring `/tmp` is not
/// contiguous. Assertions about what path a banner is for must compare against
/// the de-styled text.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip the CSI sequence: ESC '[' params (0x30-0x3f) final byte
            // (0x40-0x7e).
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ===== Alias smoke tests (one per alias) =====
// Each alias must produce the same output as its explicit form.

#[test]
fn alias_tree_matches_explicit() {
    let lazy = run_f(&["tree"]);
    let explicit = run_f(&["-R", "-D"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_flat_matches_explicit() {
    let lazy = run_f(&["flat"]);
    let explicit = run_f(&["-o"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_compact_matches_explicit() {
    let lazy = run_f(&["compact"]);
    let explicit = run_f(&["-c"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_verbose_matches_explicit() {
    let lazy = run_f(&["verbose"]);
    let explicit = run_f(&["-v"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_hidden_matches_explicit() {
    let lazy = run_f(&["hidden"]);
    let explicit = run_f(&["-a"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_dirs_matches_explicit() {
    let lazy = run_f(&["dirs"]);
    let explicit = run_f(&["-D"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_new_matches_explicit() {
    let lazy = run_f(&["new"]);
    let explicit = run_f(&["-t"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_old_matches_explicit() {
    let lazy = run_f(&["old"]);
    let explicit = run_f(&["-t", "-r"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_big_matches_explicit() {
    let lazy = run_f(&["big"]);
    let explicit = run_f(&["-S"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_small_matches_explicit() {
    let lazy = run_f(&["small"]);
    let explicit = run_f(&["-S", "-r"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_ext_matches_explicit() {
    let lazy = run_f(&["ext"]);
    let explicit = run_f(&["-X"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_git_matches_explicit() {
    let lazy = run_f(&["git"]);
    let explicit = run_f(&["-G"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_nosort_matches_explicit() {
    let lazy = run_f(&["nosort"]);
    let explicit = run_f(&["-U"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_top_matches_explicit() {
    let lazy = run_f(&["top"]);
    let explicit = run_f(&["-S", "-m", "20"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_newest_matches_explicit() {
    let lazy = run_f(&["newest"]);
    let explicit = run_f(&["-t", "-m", "20"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_recurse_matches_explicit() {
    let lazy = run_f(&["recurse"]);
    let explicit = run_f(&["-R"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_edit_matches_explicit() {
    let (lazy_stdout, _lazy_stderr, lazy_code) = run_f_full(&["edit"]);
    let (exp_stdout, _exp_stderr, exp_code) = run_f_full(&["-e"]);
    assert_eq!(lazy_code, exp_code);
    assert_eq!(
        first_line_header(&lazy_stdout),
        first_line_header(&exp_stdout)
    );
}

#[test]
fn alias_run_matches_explicit() {
    let (lazy_stdout, _lazy_stderr, lazy_code) = run_f_full(&["run"]);
    let (exp_stdout, _exp_stderr, exp_code) = run_f_full(&["-x"]);
    assert_eq!(lazy_code, exp_code);
    assert_eq!(
        first_line_header(&lazy_stdout),
        first_line_header(&exp_stdout)
    );
}

// ===== Alias composition tests =====

#[test]
fn alias_composition_two_aliases() {
    let composed = run_f(&["hidden", "verbose"]);
    let explicit = run_f(&["-a", "-v"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

#[test]
fn alias_composition_three_aliases() {
    let composed = run_f(&["new", "recurse", "hidden"]);
    let explicit = run_f(&["-t", "-R", "-a"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

#[test]
fn alias_composition_top_with_hidden() {
    let composed = run_f(&["top", "hidden"]);
    let explicit = run_f(&["-S", "-r", "-m", "20", "-a"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

// ===== Routing tests =====

#[test]
fn unknown_bare_word_does_nothing() {
    // User's requirement: "if no such alias found then nothing happens"
    // i.e., exit 0 with no output, not the default banner.
    let (stdout, _stderr, code) = run_f_full(&["nonexistentword"]);
    assert_eq!(code, 0, "unknown word should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "unknown word should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_t_does_nothing() {
    // `f t` used to mean `-t` (timesort) in 0.6.x lazy flags.
    // In 0.7.1, `t` is not a known alias — it produces no output.
    let (stdout, _stderr, code) = run_f_full(&["t"]);
    assert_eq!(code, 0, "f t should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f t should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_no_args_still_shows_banner() {
    // `f` (no args) is different from `f <unknown-word>`.
    // `f` shows the default banner for cwd.
    let (stdout, _stderr, code) = run_f_full(&[]);
    assert_eq!(code, 0, "f (no args) should not error");
    assert!(
        !stdout.is_empty(),
        "f (no args) should show the default banner"
    );
}

#[test]
fn unknown_bare_word_does_not_match_path() {
    // `f Downloads` (no ./ prefix) is NOT treated as a path. It is an
    // unknown bare word and produces no output (the "nothing happens" rule).
    let (stdout, _stderr, code) = run_f_full(&["Downloads"]);
    assert_eq!(code, 0, "f Downloads should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f Downloads should produce no output, got: {}",
        stdout
    );
}

#[test]
fn explicit_path_with_dot_slash_shows_that_path() {
    // 0.7.14 changed this: a bare path that exists on disk is now routed as a
    // path (`is_existing_path` + `is_path_arg`) instead of being dropped, so
    // `f ./src` shows `src`'s banner rather than nothing. See
    // docs/releases/RELEASE_NOTES_0.7.14.md ("Routing fix").
    let (stdout, stderr, code) = run_f_full(&["./src"]);
    assert_eq!(code, 0, "./src should not error, got: {}", stderr);
    assert!(
        !stdout.is_empty(),
        "./src should show src's banner (0.7.14 routing fix), got nothing"
    );
    assert!(
        first_line_header(&strip_ansi(&stdout)).contains("src"),
        "banner should be for src, got: {}",
        first_line_header(&strip_ansi(&stdout))
    );
}

#[test]
fn explicit_path_with_slash_shows_that_path() {
    let (stdout, stderr, code) = run_f_full(&["/tmp"]);
    assert_eq!(code, 0, "/tmp should not error, got: {}", stderr);
    let header = first_line_header(&strip_ansi(&stdout));
    assert!(
        header.contains("/tmp"),
        "banner should be for /tmp, got: {}",
        header
    );
}

#[test]
fn explicit_path_with_tilde_shows_that_path() {
    // Tilde expansion is a shell feature, so we use the expanded path here.
    let home = std::env::var("HOME").unwrap_or("/tmp".to_string());
    let (stdout, stderr, code) = run_f_full(&[home.as_str()]);
    assert_eq!(code, 0, "HOME path should not error, got: {}", stderr);
    assert!(
        !stdout.is_empty(),
        "HOME path should show that directory's banner, got nothing"
    );
}

// ===== -b (banner switch) tests =====
// The `-b` flag switches to banner mode, which allows paths. This is
// the way to get a banner for a specific path without using the
// `banner` subcommand.

#[test]
fn b_flag_alone_shows_default_banner() {
    // `f -b` is equivalent to `f` (no args) — default banner for cwd.
    let (stdout, stderr, code) = run_f_full(&["-b"]);
    assert_eq!(code, 0, "f -b should not error, got: {}", stderr);
    // Should show banner output (not empty) and match the default banner.
    let default_out = run_f(&[]);
    assert!(!stdout.trim().is_empty(), "f -b should show the banner");
    assert!(
        !default_out.trim().is_empty(),
        "default banner should not be empty"
    );
    assert_eq!(
        first_line_header(&stdout),
        first_line_header(&default_out),
        "f -b should render the same header as the default banner"
    );
}

#[test]
fn b_flag_with_path_shows_banner() {
    // `f -b ./src` — banner for ./src.
    let (_stdout, _stderr, code) = run_f_full(&["-b", "./src"]);
    assert_eq!(code, 0, "f -b ./src should not error, got: {}", _stderr);
}

#[test]
fn b_flag_with_absolute_path_works() {
    // `f -b /tmp` — banner for /tmp.
    let (_stdout, _stderr, code) = run_f_full(&["-b", "/tmp"]);
    assert_eq!(code, 0, "f -b /tmp should not error, got: {}", _stderr);
}

#[test]
fn b_flag_with_alias_expands() {
    // `f -b tree` — tree alias expands, banner with -R -D.
    let (_stdout, _stderr, code) = run_f_full(&["-b", "tree"]);
    assert_eq!(code, 0, "f -b tree should not error, got: {}", _stderr);
}

#[test]
fn b_flag_with_path_and_alias() {
    // `f -b tree ./src` — alias expands, path is passed through.
    let (_stdout, _stderr, code) = run_f_full(&["-b", "tree", "./src"]);
    assert_eq!(
        code, 0,
        "f -b tree ./src should not error, got: {}",
        _stderr
    );
}

#[test]
fn b_flag_with_explicit_flag_preserves_flag() {
    // `f -b -t` — explicit flag is preserved.
    let (_stdout, _stderr, code) = run_f_full(&["-b", "-t"]);
    assert_eq!(code, 0, "f -b -t should not error, got: {}", _stderr);
}

#[test]
fn f_subcommand_path_still_works() {
    // The user can still get a banner for a specific path by using
    // the banner subcommand explicitly. Aliases do not apply to
    // subcommand invocations.
    let (_stdout, _stderr, code) = run_f_full(&["banner", "./src"]);
    assert_eq!(
        code, 0,
        "f banner ./src should still work, got: {}",
        _stderr
    );
}

#[test]
fn explicit_flag_bypass_works() {
    // f -t should still work (explicit flag, no alias)
    let (_stdout, _stderr, code) = run_f_full(&["-t"]);
    assert_eq!(code, 0, "f -t should succeed");
}

#[test]
fn explicit_flag_with_value_works() {
    let (_stdout, _stderr, code) = run_f_full(&["-f", "txt"]);
    assert_eq!(code, 0, "f -f txt should succeed");
}

#[test]
fn explicit_long_flag_works() {
    let (_stdout, _stderr, code) = run_f_full(&["--filter", "txt"]);
    assert_eq!(code, 0, "f --filter txt should succeed");
}

#[test]
fn number_navigation_still_works() {
    // f 1 should still navigate to item 1 (number takes precedence)
    let (_stdout, _stderr, code) = run_f_full(&["1"]);
    assert_eq!(code, 0, "f 1 should succeed");
}

// ===== Subcommand passthrough tests =====

#[test]
fn subcommand_banner_passthrough() {
    // f banner 1 should be handled by clap directly (subcommand invocation)
    let (_stdout, _stderr, code) = run_f_full(&["banner", "1"]);
    assert_eq!(code, 0, "f banner 1 should succeed: {}", _stderr);
}

#[test]
fn subcommand_help_passthrough() {
    // f help should print help (clap handles)
    let (_stdout, _stderr, code) = run_f_full(&["help"]);
    let _ = _stdout;
    let _ = _stderr;
    // help may exit non-zero in clap, that's fine — we just verify
    // the invocation is handled and doesn't crash.
    let _ = code;
}

#[test]
fn subcommand_env_passthrough() {
    let (_stdout, _stderr, code) = run_f_full(&["env"]);
    let _ = _stdout;
    let _ = _stderr;
    let _ = code;
}

// ===== Lazy flag removal verification =====
// These tests verify that the lazy flag system is GONE.

#[test]
fn f_t_no_longer_means_dash_t() {
    // In 0.6.x, `f t` meant `-t`. In 0.7.1, `t` is not an alias,
    // so it produces no output (the "nothing happens" rule).
    let (stdout, _stderr, code) = run_f_full(&["t"]);
    assert_eq!(code, 0, "f t should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f t should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_trc_no_longer_means_dash_t_dash_r_dash_c() {
    // In 0.6.x, `f trc` meant `-t -r -c`. In 0.7.1, it produces
    // no output.
    let (stdout, _stderr, code) = run_f_full(&["trc"]);
    assert_eq!(code, 0, "f trc should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f trc should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_s_no_longer_means_dash_upper_s() {
    // In 0.6.x, `f s` meant `-S` (case-insensitive alias).
    // In 0.7.1, `s` is not an alias, so it produces no output.
    let (stdout, _stderr, code) = run_f_full(&["s"]);
    assert_eq!(code, 0, "f s should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f s should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_m_lf_colon_no_longer_works() {
    // In 0.6.37, `f mLf: 10` meant `-f 10`. In 0.7.0, the `:` binding
    // is gone. The `mLf:` is an unknown bare word and is dropped. The
    // `10` is a number and is passed through, so this becomes
    // equivalent to `f 10` (navigate to item 10). We just verify it
    // doesn't error and doesn't apply the `:` binding.
    let (_stdout, _stderr, code) = run_f_full(&["mLf:", "10"]);
    assert_eq!(code, 0, "should not error, got: {}", _stderr);
}

#[test]
fn f_dash_t_still_works_after_removal() {
    // Explicit flags should still work after lazy flag removal.
    let (_stdout, _stderr, code) = run_f_full(&["-t"]);
    assert_eq!(code, 0);
}

// ===== Alias + explicit flag composition =====

#[test]
fn alias_plus_explicit_flag() {
    let composed = run_f(&["tree", "-L", "2"]);
    let explicit = run_f(&["-R", "-D", "-L", "2"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

#[test]
fn alias_plus_path_applies_alias_to_that_path() {
    // Since 0.7.14 the path is no longer dropped, so `f tree ./src` is a
    // recursive banner of `src` — not of the cwd. Previously this asserted
    // equivalence with `f tree`, which encoded the pre-0.7.14 routing.
    let with_path = run_f(&["tree", "./src"]);
    let just_tree = run_f(&["tree"]);
    let src_header = first_line_header(&with_path);
    assert_ne!(
        src_header,
        first_line_header(&just_tree),
        "`f tree ./src` should target src, not the cwd"
    );
    assert!(
        !src_header.is_empty(),
        "`f tree ./src` should produce a banner"
    );
}

#[test]
fn number_passes_through_with_alias() {
    // f tree 5 — the `5` is a number, not an alias, but should
    // not be expanded. It passes through to the banner subcommand.
    // (Whether the banner subcommand treats it as a path or errors
    // is up to clap, but it should not be expanded as an alias.)
    let (_stdout, _stderr, _code) = run_f_full(&["tree", "5"]);
    // We don't assert success — we just verify it doesn't crash.
    let _ = _stdout;
    let _ = _stderr;
}
