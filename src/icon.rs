//! File-type icon system — 3-tier lookup: exact name → extension → type fallback
//!
//! Supports two modes controlled by CFM_ICONS env var:
//! - "emoji" (default): emoji icons (works everywhere)
//! - "nerd": Nerd Font glyphs (requires terminal with Nerd Font support)

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

/// Exact filename → icon
fn exact_name_icon(name: &str, use_nerd: bool) -> Option<&'static str> {
    match name {
        // Project files
        "cargo.toml" => Some(emoji_or_nerd("🦀", "\u{f43b}", use_nerd)),      // nf-md-language-rust
        "cargo.lock" | "package-lock.json" | "bun.lock" | "yarn.lock" | "pnpm-lock.yaml" => {
            Some(emoji_or_nerd("🔒", "\u{f023}", use_nerd))                  // nf-fa-lock
        }
        "package.json" => Some(emoji_or_nerd("📦", "\u{f4e6}", use_nerd)),    // nf-md-package-variant
        "go.mod" | "go.sum" => Some(emoji_or_nerd("🐹", "\u{e627}", use_nerd)), // nf-dev-go
        "pyproject.toml" | "setup.py" | "requirements.txt" | "pipfile" | "pyrightconfig.json" => {
            Some(emoji_or_nerd("🐍", "\u{e73c}", use_nerd))                  // nf-md-language-python
        }
        "gemfile" => Some(emoji_or_nerd("💎", "\u{e219}", use_nerd)),         // nf-mdi-gem
        "pom.xml" | "build.gradle" => Some(emoji_or_nerd("☕", "\u{e268}", use_nerd)), // nf-dev-java
        "cmakelists.txt" | "meson.build" => Some(emoji_or_nerd("⚙️", "\u{e995}", use_nerd)), // nf-md-cog

        // Build / CI
        "makefile" => Some(emoji_or_nerd("🔨", "\u{ea3a}", use_nerd)),       // nf-mdi-hammer
        "dockerfile" => Some(emoji_or_nerd("🐳", "\u{f308}", use_nerd)),     // nf-dev-docker
        ".dockerignore" => Some(emoji_or_nerd("🐳", "\u{f308}", use_nerd)),
        ".gitignore" | ".gitattributes" | ".gitmodules" => Some(emoji_or_nerd("📋", "\u{fc71}", use_nerd)), // nf-md-file-document-outline

        // Config
        ".env" | ".env.local" | ".env.production" => Some(emoji_or_nerd("🔐", "\u{f023}", use_nerd)),
        ".editorconfig" | ".prettierrc" | ".eslintrc" | ".eslintrc.json" | ".eslintrc.js" => {
            Some(emoji_or_nerd("📝", "\u{f4a3}", use_nerd))                  // nf-mdi-file-document-edit-outline
        }
        "tsconfig.json" | "webpack.config.js" | "vite.config.js" | "vite.config.ts" => {
            Some(emoji_or_nerd("📝", "\u{f4a3}", use_nerd))
        }
        "tailwind.config.js" | "tailwind.config.ts" | "next.config.js" | "next.config.mjs" => {
            Some(emoji_or_nerd("📝", "\u{f4a3}", use_nerd))
        }
        "nuxt.config.ts" | "svelte.config.js" | "astro.config.mjs" => {
            Some(emoji_or_nerd("📝", "\u{f4a3}", use_nerd))
        }

        // READMEs / docs
        "readme.md" | "readme.rst" | "readme.txt" => Some(emoji_or_nerd("📖", "\u{f72b}", use_nerd)), // nf-md-file-document-outline
        "changelog.md" | "changelog" => Some(emoji_or_nerd("📋", "\u{f72b}", use_nerd)),
        "license" | "license.md" | "license.txt" => Some(emoji_or_nerd("📜", "\u{f719}", use_nerd)),  // nf-md-file-certificate-outline
        "todo.md" => Some(emoji_or_nerd("📝", "\u{f4a3}", use_nerd)),

        // Nix
        "flake.nix" | "flake.lock" | "shell.nix" | "default.nix" => {
            Some(emoji_or_nerd("❄️", "\u{e2a1}", use_nerd))                  // nf-md-snowflake
        }

        _ => None,
    }
}

/// Extension → icon
fn extension_icon(ext: &str, use_nerd: bool) -> Option<&'static str> {
    match ext {
        // Languages
        ".rs" => Some(emoji_or_nerd("🦀", "\u{f43b}", use_nerd)),
        ".py" => Some(emoji_or_nerd("🐍", "\u{e73c}", use_nerd)),
        ".js" | ".mjs" | ".cjs" => Some(emoji_or_nerd("📜", "\u{f723}", use_nerd)),           // nf-md-file-js
        ".ts" => Some(emoji_or_nerd("📘", "\u{e628}", use_nerd)),                             // nf-dev-typescript
        ".jsx" | ".tsx" => Some(emoji_or_nerd("⚛️", "\u{e625}", use_nerd)),                    // nf-dev-react
        ".go" => Some(emoji_or_nerd("🐹", "\u{e627}", use_nerd)),
        ".rb" => Some(emoji_or_nerd("💎", "\u{e219}", use_nerd)),
        ".java" => Some(emoji_or_nerd("☕", "\u{e268}", use_nerd)),
        ".kt" => Some(emoji_or_nerd("🟣", "\u{e628}", use_nerd)),                             // nf-md-language-kotlin
        ".c" | ".h" => Some(emoji_or_nerd("🔧", "\u{f12f}", use_nerd)),                       // nf-mdi-file-code-outline
        ".cpp" | ".hpp" => Some(emoji_or_nerd("⚙️", "\u{e995}", use_nerd)),
        ".cs" => Some(emoji_or_nerd("🎮", "\u{e68f}", use_nerd)),                             // nf-mdi-language-csharp
        ".swift" => Some(emoji_or_nerd("🐦", "\u{e755}", use_nerd)),                           // nf-md-language-swift
        ".zig" => Some(emoji_or_nerd("⚡", "\u{e0e9}", use_nerd)),                             // nf-md-lightning-bolt
        ".nim" | ".nims" => Some(emoji_or_nerd("🌙", "\u{e73c}", use_nerd)),
        ".lua" => Some(emoji_or_nerd("🌙", "\u{e73c}", use_nerd)),
        ".vim" | ".el" => Some(emoji_or_nerd("📝", "\u{f4a3}", use_nerd)),
        ".r" | ".R" => Some(emoji_or_nerd("📊", "\u{f200}", use_nerd)),                        // nf-md-chart-box-outline
        ".scala" => Some(emoji_or_nerd("🔴", "\u{e68f}", use_nerd)),
        ".ex" | ".exs" => Some(emoji_or_nerd("🟣", "\u{e7a1}", use_nerd)),                    // nf-md-language-elixir
        ".erl" => Some(emoji_or_nerd("🟣", "\u{e7a1}", use_nerd)),
        ".hs" | ".ml" | ".fs" => Some(emoji_or_nerd("🟣", "\u{e61f}", use_nerd)),              // nf-dev-haskell

        // Web
        ".html" | ".htm" => Some(emoji_or_nerd("🌐", "\u{f6a8}", use_nerd)),                   // nf-md-language-html
        ".css" | ".scss" | ".sass" | ".less" => Some(emoji_or_nerd("🎨", "\u{f368}", use_nerd)), // nf-mdi-language-css3
        ".vue" => Some(emoji_or_nerd("💚", "\u{f584}", use_nerd)),                              // nf-md-vuejs
        ".svelte" => Some(emoji_or_nerd("🧡", "\u{e73c}", use_nerd)),

        // Config / data
        ".json" => Some(emoji_or_nerd("📋", "\u{f724}", use_nerd)),                            // nf-md-code-json
        ".yaml" | ".yml" => Some(emoji_or_nerd("📋", "\u{f724}", use_nerd)),
        ".toml" => Some(emoji_or_nerd("📋", "\u{f724}", use_nerd)),
        ".xml" => Some(emoji_or_nerd("📋", "\u{f724}", use_nerd)),
        ".ini" | ".conf" | ".cfg" => Some(emoji_or_nerd("📋", "\u{f724}", use_nerd)),

        // Shell / scripts
        ".sh" | ".bash" | ".zsh" | ".fish" => Some(emoji_or_nerd("🐚", "\u{e795}", use_nerd)), // nf-dev-terminal
        ".ps1" | ".bat" | ".cmd" => Some(emoji_or_nerd("🐚", "\u{e795}", use_nerd)),

        // Docs / text
        ".md" | ".rst" => Some(emoji_or_nerd("📖", "\u{f72b}", use_nerd)),                     // nf-md-file-document-outline
        ".txt" => Some(emoji_or_nerd("📄", "\u{f15c}", use_nerd)),                             // nf-fa-file-o
        ".pdf" => Some(emoji_or_nerd("📕", "\u{f724}", use_nerd)),
        ".doc" | ".docx" | ".odt" => Some(emoji_or_nerd("📘", "\u{f724}", use_nerd)),

        // Images
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".svg" | ".webp" | ".ico" | ".bmp" | ".tiff" | ".avif" => {
            Some(emoji_or_nerd("🖼️", "\u{f1c5}", use_nerd))                                   // nf-fa-file-image-o
        }

        // Audio
        ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".m4a" => {
            Some(emoji_or_nerd("🎵", "\u{f1c7}", use_nerd))                                   // nf-fa-file-audio-o
        }

        // Video
        ".mp4" | ".mkv" | ".avi" | ".mov" | ".webm" => {
            Some(emoji_or_nerd("🎬", "\u{f1c8}", use_nerd))                                   // nf-fa-file-video-o
        }

        // Archives
        ".zip" | ".tar" | ".gz" | ".bz2" | ".xz" | ".7z" | ".rar" | ".tgz" => {
            Some(emoji_or_nerd("📦", "\u{f187}", use_nerd))                                   // nf-fa-file-archive-o
        }

        // Binary / build
        ".o" | ".so" | ".dll" | ".dylib" | ".a" => Some(emoji_or_nerd("🔧", "\u{f12f}", use_nerd)),
        ".exe" | ".bin" | ".wasm" => Some(emoji_or_nerd("⚡", "\u{e0e9}", use_nerd)),

        // Database
        ".db" | ".sqlite" | ".sqlite3" => Some(emoji_or_nerd("🗃️", "\u{f1c0}", use_nerd)),     // nf-fa-database

        // Lock files
        ".lock" => Some(emoji_or_nerd("🔒", "\u{f023}", use_nerd)),

        // Nix
        ".nix" => Some(emoji_or_nerd("❄️", "\u{e2a1}", use_nerd)),

        _ => None,
    }
}

fn emoji_or_nerd(emoji: &str, nerd: &str, use_nerd: bool) -> String {
    if use_nerd { nerd.to_string() } else { emoji.to_string() }
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
        // Just verify nerd mode doesn't panic and returns something
        let result = icon_for("Cargo.toml", false, false, false);
        assert!(!result.is_empty());
        std::env::remove_var("CFM_ICONS");
    }
}