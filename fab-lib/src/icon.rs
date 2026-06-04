//! File-type icon system — 3-tier lookup: exact name → extension → type fallback
//!
//! Supports two modes controlled by FAB_ICONS env var:
//! - "emoji" (default): emoji icons (works everywhere)
//! - "nerd": Nerd Font glyphs (requires terminal with Nerd Font support)

/// Get icon for a file/directory entry, reading mode from FAB_ICONS env var
pub fn icon_for(name: &str, is_dir: bool, is_exec: bool, is_symlink: bool) -> String {
    let use_nerd = std::env::var("FAB_ICONS")
        .map(|v| v == "nerd")
        .unwrap_or(false);
    icon_for_mode(name, is_dir, is_exec, is_symlink, use_nerd)
}

/// Get icon for a file/directory entry with explicit nerd mode
pub fn icon_for_mode(name: &str, is_dir: bool, is_exec: bool, is_symlink: bool, use_nerd: bool) -> String {

    if is_symlink {
        return "🔗".to_string();
    }
    if is_dir {
        return "📁".to_string();
    }

    let name_lower = name.to_lowercase();

    // Tier 1: exact filename match
    if let Some(icon) = exact_name_icon(&name_lower, use_nerd) {
        return icon;
    }

    // Tier 2: extension match
    if let Some(dot_pos) = name_lower.rfind('.') {
        let ext = &name_lower[dot_pos..];
        if let Some(icon) = extension_icon(ext, use_nerd) {
            return icon;
        }
    }

    // Tier 3: type fallback
    if is_exec {
        return "⚡".to_string();
    }
    "📄".to_string()
}

/// Exact filename → icon (returns owned String)
fn exact_name_icon(name: &str, use_nerd: bool) -> Option<String> {
    match name {
        // Project files
        "cargo.toml" => Some(nerd_or_emoji("\u{f43b}", "🦀", use_nerd)),
        "cargo.lock" | "package-lock.json" | "bun.lock" | "yarn.lock" | "pnpm-lock.yaml" => {
            Some(nerd_or_emoji("\u{f023}", "🔒", use_nerd))
        }
        "package.json" => Some(nerd_or_emoji("\u{f4e6}", "📦", use_nerd)),
        "go.mod" | "go.sum" => Some(nerd_or_emoji("\u{e627}", "🐹", use_nerd)),
        "pyproject.toml" | "setup.py" | "requirements.txt" | "pipfile" | "pyrightconfig.json" => {
            Some(nerd_or_emoji("\u{e73c}", "🐍", use_nerd))
        }
        "gemfile" => Some(nerd_or_emoji("\u{e219}", "💎", use_nerd)),
        "pom.xml" | "build.gradle" => Some(nerd_or_emoji("\u{e268}", "☕", use_nerd)),
        "cmakelists.txt" | "meson.build" => Some(nerd_or_emoji("\u{e995}", "⚙️", use_nerd)),

        // Build / CI
        "makefile" => Some(nerd_or_emoji("\u{ea3a}", "🔨", use_nerd)),
        "dockerfile" => Some(nerd_or_emoji("\u{f308}", "🐳", use_nerd)),
        ".dockerignore" => Some(nerd_or_emoji("\u{f308}", "🐳", use_nerd)),
        ".gitignore" | ".gitattributes" | ".gitmodules" => {
            Some(nerd_or_emoji("\u{f72b}", "📋", use_nerd))
        }

        // Config
        ".env" | ".env.local" | ".env.production" => {
            Some(nerd_or_emoji("\u{f023}", "🔐", use_nerd))
        }
        ".editorconfig" | ".prettierrc" | ".eslintrc" | ".eslintrc.json" | ".eslintrc.js" => {
            Some(nerd_or_emoji("\u{f4a3}", "📝", use_nerd))
        }
        "tsconfig.json" | "webpack.config.js" | "vite.config.js" | "vite.config.ts" => {
            Some(nerd_or_emoji("\u{f4a3}", "📝", use_nerd))
        }
        "tailwind.config.js" | "tailwind.config.ts" | "next.config.js" | "next.config.mjs" => {
            Some(nerd_or_emoji("\u{f4a3}", "📝", use_nerd))
        }
        "nuxt.config.ts" | "svelte.config.js" | "astro.config.mjs" => {
            Some(nerd_or_emoji("\u{f4a3}", "📝", use_nerd))
        }

        // READMEs / docs
        "readme.md" | "readme.rst" | "readme.txt" => {
            Some(nerd_or_emoji("\u{f72b}", "📖", use_nerd))
        }
        "changelog.md" | "changelog" => Some(nerd_or_emoji("\u{f72b}", "📋", use_nerd)),
        "license" | "license.md" | "license.txt" => Some(nerd_or_emoji("\u{f719}", "📜", use_nerd)),
        "todo.md" => Some(nerd_or_emoji("\u{f4a3}", "📝", use_nerd)),

        // Nix
        "flake.nix" | "flake.lock" | "shell.nix" | "default.nix" => {
            Some(nerd_or_emoji("\u{e2a1}", "❄️", use_nerd))
        }

        _ => None,
    }
}

/// Extension → icon (returns owned String)
fn extension_icon(ext: &str, use_nerd: bool) -> Option<String> {
    match ext {
        // Languages
        ".rs" => Some(nerd_or_emoji("\u{f43b}", "🦀", use_nerd)),
        ".py" => Some(nerd_or_emoji("\u{e73c}", "🐍", use_nerd)),
        ".js" | ".mjs" | ".cjs" => Some(nerd_or_emoji("\u{f723}", "📜", use_nerd)),
        ".ts" => Some(nerd_or_emoji("\u{e628}", "📘", use_nerd)),
        ".jsx" | ".tsx" => Some(nerd_or_emoji("\u{e625}", "⚛️", use_nerd)),
        ".go" => Some(nerd_or_emoji("\u{e627}", "🐹", use_nerd)),
        ".rb" => Some(nerd_or_emoji("\u{e219}", "💎", use_nerd)),
        ".java" => Some(nerd_or_emoji("\u{e268}", "☕", use_nerd)),
        ".kt" => Some(nerd_or_emoji("\u{e628}", "🟣", use_nerd)),
        ".c" | ".h" => Some(nerd_or_emoji("\u{f12f}", "🔧", use_nerd)),
        ".cpp" | ".hpp" => Some(nerd_or_emoji("\u{e995}", "⚙️", use_nerd)),
        ".cs" => Some(nerd_or_emoji("\u{e68f}", "🎮", use_nerd)),
        ".swift" => Some(nerd_or_emoji("\u{e755}", "🐦", use_nerd)),
        ".zig" => Some(nerd_or_emoji("\u{e0e9}", "⚡", use_nerd)),
        ".nim" | ".nims" => Some(nerd_or_emoji("\u{e73c}", "🌙", use_nerd)),
        ".lua" => Some(nerd_or_emoji("\u{e73c}", "🌙", use_nerd)),
        ".vim" | ".el" => Some(nerd_or_emoji("\u{f4a3}", "📝", use_nerd)),
        ".r" | ".R" => Some(nerd_or_emoji("\u{f200}", "📊", use_nerd)),
        ".scala" => Some(nerd_or_emoji("\u{e68f}", "🔴", use_nerd)),
        ".ex" | ".exs" => Some(nerd_or_emoji("\u{e7a1}", "🟣", use_nerd)),
        ".erl" => Some(nerd_or_emoji("\u{e7a1}", "🟣", use_nerd)),
        ".hs" | ".ml" | ".fs" => Some(nerd_or_emoji("\u{e61f}", "🟣", use_nerd)),

        // Web
        ".html" | ".htm" => Some(nerd_or_emoji("\u{f6a8}", "🌐", use_nerd)),
        ".css" | ".scss" | ".sass" | ".less" => Some(nerd_or_emoji("\u{f368}", "🎨", use_nerd)),
        ".vue" => Some(nerd_or_emoji("\u{f584}", "💚", use_nerd)),
        ".svelte" => Some(nerd_or_emoji("\u{e73c}", "🧡", use_nerd)),

        // Config / data
        ".json" => Some(nerd_or_emoji("\u{f724}", "📋", use_nerd)),
        ".yaml" | ".yml" => Some(nerd_or_emoji("\u{f724}", "📋", use_nerd)),
        ".toml" => Some(nerd_or_emoji("\u{f724}", "📋", use_nerd)),
        ".xml" => Some(nerd_or_emoji("\u{f724}", "📋", use_nerd)),
        ".ini" | ".conf" | ".cfg" => Some(nerd_or_emoji("\u{f724}", "📋", use_nerd)),

        // Shell / scripts
        ".sh" | ".bash" | ".zsh" | ".fish" => Some(nerd_or_emoji("\u{e795}", "🐚", use_nerd)),
        ".ps1" | ".bat" | ".cmd" => Some(nerd_or_emoji("\u{e795}", "🐚", use_nerd)),

        // Docs / text
        ".md" | ".rst" => Some(nerd_or_emoji("\u{f72b}", "📖", use_nerd)),
        ".txt" => Some(nerd_or_emoji("\u{f15c}", "📄", use_nerd)),
        ".pdf" => Some(nerd_or_emoji("\u{f724}", "📕", use_nerd)),
        ".doc" | ".docx" | ".odt" => Some(nerd_or_emoji("\u{f724}", "📘", use_nerd)),

        // Images
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".svg" | ".webp" | ".ico" | ".bmp" | ".tiff"
        | ".avif" => Some(nerd_or_emoji("\u{f1c5}", "🖼️", use_nerd)),

        // Audio
        ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".m4a" => {
            Some(nerd_or_emoji("\u{f1c7}", "🎵", use_nerd))
        }

        // Video
        ".mp4" | ".mkv" | ".avi" | ".mov" | ".webm" => {
            Some(nerd_or_emoji("\u{f1c8}", "🎬", use_nerd))
        }

        // Archives
        ".zip" | ".tar" | ".gz" | ".bz2" | ".xz" | ".7z" | ".rar" | ".tgz" => {
            Some(nerd_or_emoji("\u{f187}", "📦", use_nerd))
        }

        // Binary / build
        ".o" | ".so" | ".dll" | ".dylib" | ".a" => Some(nerd_or_emoji("\u{f12f}", "🔧", use_nerd)),
        ".exe" | ".bin" | ".wasm" => Some(nerd_or_emoji("\u{e0e9}", "⚡", use_nerd)),

        // Database
        ".db" | ".sqlite" | ".sqlite3" => Some(nerd_or_emoji("\u{f1c0}", "🗃️", use_nerd)),

        // Lock files
        ".lock" => Some(nerd_or_emoji("\u{f023}", "🔒", use_nerd)),

        // Nix
        ".nix" => Some(nerd_or_emoji("\u{e2a1}", "❄️", use_nerd)),

        _ => None,
    }
}

fn nerd_or_emoji(nerd: &str, emoji: &str, use_nerd: bool) -> String {
    if use_nerd {
        nerd.to_string()
    } else {
        emoji.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_name() {
        assert_eq!(icon_for_mode("Cargo.toml", false, false, false, false), "🦀");
        assert_eq!(icon_for_mode("package.json", false, false, false, false), "📦");
        assert_eq!(icon_for_mode("README.md", false, false, false, false), "📖");
    }

    #[test]
    fn test_extension() {
        assert_eq!(icon_for_mode("main.rs", false, false, false, false), "🦀");
        assert_eq!(icon_for_mode("app.py", false, false, false, false), "🐍");
        assert_eq!(icon_for_mode("index.html", false, false, false, false), "🌐");
        assert_eq!(icon_for_mode("style.css", false, false, false, false), "🎨");
    }

    #[test]
    fn test_fallback() {
        assert_eq!(icon_for_mode("foo", false, false, false, false), "📄");
        assert_eq!(icon_for_mode("foo", false, true, false, false), "⚡");
    }

    #[test]
    fn test_nerd_mode() {
        let result = icon_for_mode("Cargo.toml", false, false, false, true);
        assert!(!result.is_empty());
        assert_ne!(result, "🦀"); // nerd glyph differs from emoji
    }
}
