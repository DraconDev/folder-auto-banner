        if git_info.is_repo {
            // Row 1: Path + Git details (explicit)
            let mut parts = vec![format!("{} {}", project_icon, path_display)];
            if !branch_display.is_empty() {
                parts.push(format!("{}{}{}", color(BOLD), branch_display, color(RESET)));
            }
            if let Some(ref tag) = git_info.tag {
                parts.push(format!("{}{}{}", color(YELLOW), tag, color(RESET)));
            }
            // Git status indicators
            if !git_status_str.is_empty() {
                parts.push(git_status_str.clone());
            }
            // Last commit time
            if let Some(time) = git_info.last_commit_time {
                let now = chrono::Utc::now().timestamp();
                let diff = now - time;
                let time_str = if diff < 60 {
                    "just now".to_string()
                } else if diff < 3600 {
                    format!("{}m ago", diff / 60)
                } else if diff < 86400 {
                    format!("{}h ago", diff / 3600)
                } else {
                    format!("{}d ago", diff / 86400)
                };
                parts.push(format!("last {}", time_str));
            }
            // Commits today
            if git_info.commits_today > 0 {
                parts.push(format!("{} today", git_info.commits_today));
            }
            // Diff stats
            if git_info.lines_added > 0 || git_info.lines_deleted > 0 {
                parts.push(format!("+{} -{}", git_info.lines_added, git_info.lines_deleted));
            }
            let row1 = parts.join(" │ ");

            // Row 2: Stats with labels
            let mut details = Vec::new();
            
            // File stats
            details.push(format!("{}💾 {} total{}", color(CYAN), size_str, color(RESET)));
            details.push(format!("{}📄 {} files{}", color(DIM), summary.files, color(RESET)));
            details.push(format!("{}📂 {} dirs{}", color(DIM), summary.dirs, color(RESET)));
            
            // Code metrics
            if let Some(ref todos) = summary.todo_info {
                if todos.count > 0 {
                    details.push(format!("{}📝 {} TODOs{}", color(YELLOW), todos.count, color(RESET)));
                }
            }
            if let Some(ref metrics) = summary.code_metrics {
                if metrics.total_loc > 0 {
                    let loc_str = format_loc(metrics.total_loc);
                    details.push(format!("{}📊 {} lines{}", color(GREEN), loc_str, color(RESET)));
                    // Show top 3 languages
                    if !metrics.by_extension.is_empty() && metrics.total_loc > 0 {
                        let lang_parts: Vec<String> = metrics
                            .by_extension
                            .iter()
                            .take(3)
                            .map(|(ext, loc)| {
                                let pct = (*loc as f64 / metrics.total_loc as f64 * 100.0) as usize;
                                let name = match ext.as_str() {
                                    "rs" => "Rust",
                                    "md" | "mdx" => "Markdown",
                                    "sh" | "bash" => "Shell",
                                    "py" => "Python",
                                    "js" | "mjs" => "JavaScript",
                                    "ts" | "tsx" => "TypeScript",
                                    "go" => "Go",
                                    "c" | "h" => "C",
                                    "cpp" | "cc" | "cxx" | "hpp" => "C++",
                                    "java" => "Java",
                                    "rb" => "Ruby",
                                    "toml" => "TOML",
                                    "yaml" | "yml" => "YAML",
                                    "json" => "JSON",
                                    "html" | "htm" => "HTML",
                                    "css" => "CSS",
                                    "sql" => "SQL",
                                    "vim" => "VimL",
                                    "el" => "Emacs Lisp",
                                    _ => ext,
                                };
                                format!("{}{} {}%{}", color(DIM), name, pct, color(RESET))
                            })
                            .collect();
                        // Add crab icon before first language
                        if !lang_parts.is_empty() {
                            lang_parts[0] = format!("{}{}{}", project_icon, lang_parts[0], color(RESET));
                        }
                        details.push(format!("{}{}", color(DIM), lang_parts.join(" ")));
                    }
                }
            }
