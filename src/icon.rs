//! File-type icon system — 3-tier lookup: exact name → extension → type fallback

/// Get icon for a file/directory entry
pub fn icon_for(name: &str, is_dir: bool, is_exec: bool, is_symlink: bool) -> String {
    if is_symlink {
        return "🔗".to_string();
    }
    if is_dir {
        return "📁".to_string();
    }

    let name_lower = name.to_lowercase();

    // Tier 1: exact filename match
    if let Some(icon) = exact_name_icon(&name_lower) {
        return icon.to_string();
    }

    // Tier 2: extension match
    if let Some(dot_pos) = name_lower.rfind('.') {
        let ext = &name_lower[dot_pos..];
        if let Some(icon) = extension_icon(ext) {
            return icon.to_string();
        }
    }

    // Tier 3: type fallback
    if is_exec {
        return "⚡".to_string();
    }
    "📄".to_string()
}

/// Exact filename → icon
fn exact_name_icon(name: &str) -> Option<&'static str> {
    match name {
        // Project files
        "cargo.toml" => Some("🦀"),
        "cargo.lock" => Some("🔒"),
        "package.json" => Some("📦"),
        "package-lock.json" => Some("🔒"),
        "bun.lock" => Some("🔒"),
        "yarn.lock" => Some("🔒"),
        "pom.xml" => Some("☕"),
        "build.gradle" => Some("☕"),
        "go.mod" => Some("🐹"),
        "go.sum" => Some("🐹"),
        "pyproject.toml" => Some("🐍"),
        "setup.py" => Some("🐍"),
        "requirements.txt" => Some("🐍"),
        "pipfile" => Some("🐍"),
        "gemfile" => Some("💎"),
        "cmakelists.txt" => Some("⚙️"),
        "meson.build" => Some("⚙️"),

        // Build / CI
        "makefile" => Some("🔨"),
        "dockerfile" => Some("🐳"),
        ".dockerignore" => Some("🐳"),
        ".gitignore" => Some("📋"),
        ".gitattributes" => Some("📋"),
        ".gitmodules" => Some("📋"),

        // Config
        ".env" => Some("🔐"),
        ".env.local" => Some("🔐"),
        ".env.production" => Some("🔐"),
        ".editorconfig" => Some("📝"),
        ".prettierrc" => Some("📝"),
        ".eslintrc" => Some("📝"),
        ".eslintrc.json" => Some("📝"),
        ".eslintrc.js" => Some("📝"),
        "tsconfig.json" => Some("📝"),
        "webpack.config.js" => Some("📝"),
        "vite.config.js" => Some("📝"),
        "vite.config.ts" => Some("📝"),
        "tailwind.config.js" => Some("📝"),
        "tailwind.config.ts" => Some("📝"),
        "next.config.js" => Some("📝"),
        "next.config.mjs" => Some("📝"),
        "nuxt.config.ts" => Some("📝"),
        "svelte.config.js" => Some("📝"),
        "astro.config.mjs" => Some("📝"),

        // READMEs / docs
        "readme.md" => Some("📖"),
        "readme.rst" => Some("📖"),
        "readme.txt" => Some("📖"),
        "changelog.md" => Some("📋"),
        "changelog" => Some("📋"),
        "license" => Some("📜"),
        "license.md" => Some("📜"),
        "license.txt" => Some("📜"),
        "todo.md" => Some("📝"),

        // Nix
        "flake.nix" => Some("❄️"),
        "flake.lock" => Some("🔒"),
        "shell.nix" => Some("❄️"),
        "default.nix" => Some("❄️"),

        _ => None,
    }
}

/// Extension → icon
fn extension_icon(ext: &str) -> Option<&'static str> {
    match ext {
        // Languages
        ".rs" => Some("🦀"),
        ".py" => Some("🐍"),
        ".js" | ".mjs" | ".cjs" => Some("📜"),
        ".ts" => Some("📘"),
        ".jsx" | ".tsx" => Some("⚛️"),
        ".go" => Some("🐹"),
        ".rb" => Some("💎"),
        ".java" => Some("☕"),
        ".kt" => Some("🟣"),
        ".c" | ".h" => Some("🔧"),
        ".cpp" | ".hpp" => Some("⚙️"),
        ".cs" => Some("🎮"),
        ".swift" => Some("🐦"),
        ".zig" => Some("⚡"),
        ".nim" => Some("🌙"),
        ".lua" => Some("🌙"),
        ".vim" => Some("📝"),
        ".el" => Some("📝"),
        ".r" | ".R" => Some("📊"),
        ".scala" => Some("🔴"),
        ".ex" | ".exs" | ".erl" => Some("🟣"),
        ".hs" | ".ml" | ".fs" => Some("🟣"),

        // Web
        ".html" | ".htm" => Some("🌐"),
        ".css" | ".scss" | ".sass" | ".less" => Some("🎨"),
        ".vue" => Some("💚"),
        ".svelte" => Some("🧡"),

        // Config / data
        ".json" => Some("📋"),
        ".yaml" | ".yml" => Some("📋"),
        ".toml" => Some("📋"),
        ".xml" => Some("📋"),
        ".ini" | ".conf" | ".cfg" => Some("📋"),

        // Shell / scripts
        ".sh" | ".bash" | ".zsh" | ".fish" | ".ps1" | ".bat" | ".cmd" => Some("🐚"),

        // Docs / text
        ".md" | ".rst" => Some("📖"),
        ".txt" => Some("📄"),
        ".pdf" => Some("📕"),
        ".doc" | ".docx" | ".odt" => Some("📘"),

        // Images
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".svg" | ".webp" | ".ico" | ".bmp" | ".tiff" | ".avif" => {
            Some("🖼️")
        }

        // Audio
        ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".m4a" => Some("🎵"),

        // Video
        ".mp4" | ".mkv" | ".avi" | ".mov" | ".webm" => Some("🎬"),

        // Archives
        ".zip" | ".tar" | ".gz" | ".bz2" | ".xz" | ".7z" | ".rar" | ".tgz" => Some("📦"),

        // Binary / build
        ".o" | ".so" | ".dll" | ".dylib" | ".a" => Some("🔧"),
        ".exe" | ".bin" | ".wasm" => Some("⚡"),

        // Database
        ".db" | ".sqlite" | ".sqlite3" => Some("🗃️"),

        // Lock files
        ".lock" => Some("🔒"),

        // Nix
        ".nix" => Some("❄️"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_name() {
        assert_eq!(icon_for("Cargo.toml", false, false, false), "🦀");
        assert_eq!(icon_for("package.json", false, false, false), "📦");
        assert_eq!(icon_for("README.md", false, false, false), "📖");
    }

    #[test]
    fn test_extension() {
        assert_eq!(icon_for("main.rs", false, false, false), "🦀");
        assert_eq!(icon_for("app.py", false, false, false), "🐍");
        assert_eq!(icon_for("index.html", false, false, false), "🌐");
        assert_eq!(icon_for("style.css", false, false, false), "🎨");
    }

    #[test]
    fn test_fallback() {
        assert_eq!(icon_for("foo", false, false, false), "📄");
        assert_eq!(icon_for("foo", false, true, false), "⚡");
        assert_eq!(icon_for("foo", true, false, false), "📁");
        assert_eq!(icon_for("foo", false, false, true), "🔗");
    }
}
