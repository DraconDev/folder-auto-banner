//! File-type icon system — 3-tier lookup: exact name → extension → type fallback
//!
//! Supports two modes controlled by CFM_ICONS env var:
//! - "emoji" (default): emoji icons
//! - "nerd": Nerd Font glyphs (requires terminal support)

/// Get icon for a file/directory entry
pub fn icon_for(name: &str, is_dir: bool, is_exec: bool, is_symlink: bool) -> String {
    let use_nerd = std::env::var("CFM_ICONS")
        .map(|v| v == "nerd")
        .unwrap_or(false);

    if is_symlink {
        return if use_nerd { "🔗" } else { "🔗" }.to_string();
    }
    if is_dir {
        return if use_nerd { "📁" } else { "📁" }.to_string();
    }

    let name_lower = name.to_lowercase();

    // Tier 1: exact filename match
    if let Some(icon) = exact_name_icon(&name_lower, use_nerd) {
        return icon.to_string();
    }

    // Tier 2: extension match
    if let Some(dot_pos) = name_lower.rfind('.') {
        let ext = &name_lower[dot_pos..];
        if let Some(icon) = extension_icon(ext, use_nerd) {
            return icon.to_string();
        }
    }

    // Tier 3: type fallback
    if is_exec {
        return if use_nerd { "⚡" } else { "⚡" }.to_string();
    }
    if use_nerd { "📄" } else { "📄" }.to_string()
}

/// Exact filename → icon (nerd, emoji)
fn exact_name_icon(name: &str, use_nerd: bool) -> Option<&'static str> {
    match name {
        // Project files
        "cargo.toml" => Some(if use_nerd { "🦀" } else { "🦀" }),
        "cargo.lock" => Some(if use_nerd { "🔒" } else { "🔒" }),
        "package.json" => Some(if use_nerd { "📦" } else { "📦" }),
        "package-lock.json" => Some(if use_nerd { "🔒" } else { "🔒" }),
        "bun.lock" => Some(if use_nerd { "🔒" } else { "🔒" }),
        "yarn.lock" => Some(if use_nerd { "🔒" } else { "🔒" }),
        "pnpm-lock.yaml" => Some(if use_nerd { "🔒" } else { "🔒" }),
        "pom.xml" => Some(if use_nerd { "☕" } else { "☕" }),
        "build.gradle" => Some(if use_nerd { "☕" } else { "☕" }),
        "go.mod" => Some(if use_nerd { "🐹" } else { "🐹" }),
        "go.sum" => Some(if use_nerd { "🐹" } else { "🐹" }),
        "pyproject.toml" => Some(if use_nerd { "🐍" } else { "🐍" }),
        "setup.py" => Some(if use_nerd { "🐍" } else { "🐍" }),
        "requirements.txt" => Some(if use_nerd { "🐍" } else { "🐍" }),
        "pipfile" => Some(if use_nerd { "🐍" } else { "🐍" }),
        "gemfile" => Some(if use_nerd { "💎" } else { "💎" }),
        "cmakelists.txt" => Some(if use_nerd { "⚙️" } else { "⚙️" }),
        "meson.build" => Some(if use_nerd { "⚙️" } else { "⚙️" }),

        // Build / CI
        "makefile" => Some(if use_nerd { "🔨" } else { "🔨" }),
        "dockerfile" => Some(if use_nerd { "🐳" } else { "🐳" }),
        ".dockerignore" => Some(if use_nerd { "🐳" } else { "🐳" }),
        ".gitignore" => Some(if use_nerd { "📋" } else { "📋" }),
        ".gitattributes" => Some(if use_nerd { "📋" } else { "📋" }),
        ".gitmodules" => Some(if use_nerd { "📋" } else { "📋" }),

        // Config
        ".env" => Some(if use_nerd { "🔐" } else { "🔐" }),
        ".env.local" => Some(if use_nerd { "🔐" } else { "🔐" }),
        ".env.production" => Some(if use_nerd { "🔐" } else { "🔐" }),
        ".editorconfig" => Some(if use_nerd { "📝" } else { "📝" }),
        ".prettierrc" => Some(if use_nerd { "📝" } else { "📝" }),
        ".eslintrc" => Some(if use_nerd { "📝" } else { "📝" }),
        ".eslintrc.json" => Some(if use_nerd { "📝" } else { "📝" }),
        ".eslintrc.js" => Some(if use_nerd { "📝" } else { "📝" }),
        "tsconfig.json" => Some(if use_nerd { "📝" } else { "📝" }),
        "webpack.config.js" => Some(if use_nerd { "📝" } else { "📝" }),
        "vite.config.js" => Some(if use_nerd { "📝" } else { "📝" }),
        "vite.config.ts" => Some(if use_nerd { "📝" } else { "📝" }),
        "tailwind.config.js" => Some(if use_nerd { "📝" } else { "📝" }),
        "tailwind.config.ts" => Some(if use_nerd { "📝" } else { "📝" }),
        "next.config.js" => Some(if use_nerd { "📝" } else { "📝" }),
        "next.config.mjs" => Some(if use_nerd { "📝" } else { "📝" }),
        "nuxt.config.ts" => Some(if use_nerd { "📝" } else { "📝" }),
        "svelte.config.js" => Some(if use_nerd { "📝" } else { "📝" }),
        "astro.config.mjs" => Some(if use_nerd { "📝" } else { "📝" }),

        // READMEs / docs
        "readme.md" => Some(if use_nerd { "📖" } else { "📖" }),
        "readme.rst" => Some(if use_nerd { "📖" } else { "📖" }),
        "readme.txt" => Some(if use_nerd { "📖" } else { "📖" }),
        "changelog.md" => Some(if use_nerd { "📋" } else { "📋" }),
        "changelog" => Some(if use_nerd { "📋" } else { "📋" }),
        "license" => Some(if use_nerd { "📜" } else { "📜" }),
        "license.md" => Some(if use_nerd { "📜" } else { "📜" }),
        "license.txt" => Some(if use_nerd { "📜" } else { "📜" }),
        "todo.md" => Some(if use_nerd { "📝" } else { "📝" }),

        // Nix
        "flake.nix" => Some(if use_nerd { "❄️" } else { "❄️" }),
        "flake.lock" => Some(if use_nerd { "🔒" } else { "🔒" }),
        "shell.nix" => Some(if use_nerd { "❄️" } else { "❄️" }),
        "default.nix" => Some(if use_nerd { "❄️" } else { "❄️" }),

        _ => None,
    }
}

/// Extension → icon (nerd, emoji)
fn extension_icon(ext: &str, use_nerd: bool) -> Option<&'static str> {
    match ext {
        // Languages
        ".rs" => Some(if use_nerd { "🦀" } else { "🦀" }),
        ".py" => Some(if use_nerd { "🐍" } else { "🐍" }),
        ".js" | ".mjs" | ".cjs" => Some(if use_nerd { "📜" } else { "📜" }),
        ".ts" => Some(if use_nerd { "📘" } else { "📘" }),
        ".jsx" | ".tsx" => Some(if use_nerd { "⚛️" } else { "⚛️" }),
        ".go" => Some(if use_nerd { "🐹" } else { "🐹" }),
        ".rb" => Some(if use_nerd { "💎" } else { "💎" }),
        ".java" => Some(if use_nerd { "☕" } else { "☕" }),
        ".kt" => Some(if use_nerd { "🟣" } else { "🟣" }),
        ".c" | ".h" => Some(if use_nerd { "🔧" } else { "🔧" }),
        ".cpp" | ".hpp" => Some(if use_nerd { "⚙️" } else { "⚙️" }),
        ".cs" => Some(if use_nerd { "🎮" } else { "🎮" }),
        ".swift" => Some(if use_nerd { "🐦" } else { "🐦" }),
        ".zig" => Some(if use_nerd { "⚡" } else { "⚡" }),
        ".nim" => Some(if use_nerd { "🌙" } else { "🌙" }),
        ".lua" => Some(if use_nerd { "🌙" } else { "🌙" }),
        ".vim" => Some(if use_nerd { "📝" } else { "📝" }),
        ".el" => Some(if use_nerd { "📝" } else { "📝" }),
        ".r" | ".R" => Some(if use_nerd { "📊" } else { "📊" }),
        ".scala" => Some(if use_nerd { "🔴" } else { "🔴" }),
        ".ex" | ".exs" | ".erl" => Some(if use_nerd { "🟣" } else { "🟣" }),
        ".hs" | ".ml" | ".fs" => Some(if use_nerd { "🟣" } else { "🟣" }),

        // Web
        ".html" | ".htm" => Some(if use_nerd { "🌐" } else { "🌐" }),
        ".css" | ".scss" | ".sass" | ".less" => Some(if use_nerd { "🎨" } else { "🎨" }),
        ".vue" => Some(if use_nerd { "💚" } else { "💚" }),
        ".svelte" => Some(if use_nerd { "🧡" } else { "🧡" }),

        // Config / data
        ".json" => Some(if use_nerd { "📋" } else { "📋" }),
        ".yaml" | ".yml" => Some(if use_nerd { "📋" } else { "📋" }),
        ".toml" => Some(if use_nerd { "📋" } else { "📋" }),
        ".xml" => Some(if use_nerd { "📋" } else { "📋" }),
        ".ini" | ".conf" | ".cfg" => Some(if use_nerd { "📋" } else { "📋" }),

        // Shell / scripts
        ".sh" | ".bash" | ".zsh" | ".fish" | ".ps1" | ".bat" | ".cmd" => {
            Some(if use_nerd { "🐚" } else { "🐚" })
        }

        // Docs / text
        ".md" | ".rst" => Some(if use_nerd { "📖" } else { "📖" }),
        ".txt" => Some(if use_nerd { "📄" } else { "📄" }),
        ".pdf" => Some(if use_nerd { "📕" } else { "📕" }),
        ".doc" | ".docx" | ".odt" => Some(if use_nerd { "📘" } else { "📘" }),

        // Images
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".svg" | ".webp" | ".ico" | ".bmp" | ".tiff" | ".avif" => {
            Some(if use_nerd { "🖼️" } else { "🖼️" })
        }

        // Audio
        ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".m4a" => {
            Some(if use_nerd { "🎵" } else { "🎵" })
        }

        // Video
        ".mp4" | ".mkv" | ".avi" | ".mov" | ".webm" => {
            Some(if use_nerd { "🎬" } else { "🎬" })
        }

        // Archives
        ".zip" | ".tar" | ".gz" | ".bz2" | ".xz" | ".7z" | ".rar" | ".tgz" => {
            Some(if use_nerd { "📦" } else { "📦" })
        }

        // Binary / build
        ".o" | ".so" | ".dll" | ".dylib" | ".a" => Some(if use_nerd { "🔧" } else { "🔧" }),
        ".exe" | ".bin" | ".wasm" => Some(if use_nerd { "⚡" } else { "⚡" }),

        // Database
        ".db" | ".sqlite" | ".sqlite3" => Some(if use_nerd { "🗃️" } else { "🗃️" }),

        // Lock files
        ".lock" => Some(if use_nerd { "🔒" } else { "🔒" }),

        // Nix
        ".nix" => Some(if use_nerd { "❄️" } else { "❄️" }),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_name() {
        std::env::remove_var("CFM_ICONS");
        assert_eq!(icon_for("Cargo.toml", false, false, false), "🦀");
        assert_eq!(icon_for("package.json", false, false, false), "📦");
        assert_eq!(icon_for("README.md", false, false, false), "📖");
    }

    #[test]
    fn test_extension() {
        std::env::remove_var("CFM_ICONS");
        assert_eq!(icon_for("main.rs", false, false, false), "🦀");
        assert_eq!(icon_for("app.py", false, false, false), "🐍");
        assert_eq!(icon_for("index.html", false, false, false), "🌐");
        assert_eq!(icon_for("style.css", false, false, false), "🎨");
    }

    #[test]
    fn test_fallback() {
        std::env::remove_var("CFM_ICONS");
        assert_eq!(icon_for("foo", false, false, false), "📄");
        assert_eq!(icon_for("foo", false, true, false), "⚡");
    }

    #[test]
    fn test_nerd_mode() {
        std::env::set_var("CFM_ICONS", "nerd");
        // Same icons for now, just different mode
        assert_eq!(icon_for("Cargo.toml", false, false, false), "🦀");
        std::env::remove_var("CFM_ICONS");
    }
}