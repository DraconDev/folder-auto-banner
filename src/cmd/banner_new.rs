    println!("{}", header);
    println!("┌{}┬{}┬{}┬{}┬{}┐", "─".repeat(5), "─".repeat(32), "─".repeat(10), "─".repeat(8), "─".repeat(15));
    println!("│{:^5}│ {:<32} │ {:>10} │ {:<8} │ {:^15} │", " ", "NAME", "COUNT/SIZE", "TYPE", "MODIFIED");
    println!("├{}┼{}┼{}┼{}┼{}┤", "─".repeat(5), "─".repeat(32), "─".repeat(10), "─".repeat(8), "─".repeat(15));
    
    for item in display_items {
        if item.is_dir {
            let count = count_items_in_dir(item);
            let count_str = if count == 1 { "1" } else { &format!("{}", count) };
            let modified = item.modified.map(|dt| format_relative_time(&dt)).unwrap_or_default();
            println!("│📂│ {:<32} │ {:>5} items │ {:<8} │ {:^15} │", item.name, count_str, "[DIR]", modified);
        } else {
            let size = format_size_compact(item.size);
            let ext = get_extension_label(item);
            let modified = item.modified.map(|dt| format_relative_time(&dt)).unwrap_or_default();
            println!("│📄│ {:<32} │ {:>8} │ {:<8} │ {:^15} │", item.name, size, ext, modified);
        }
    }
    
    println!("└{}┴{}┴{}┴{}┴{}┘", "─".repeat(5), "─".repeat(32), "─".repeat(10), "─".repeat(8), "─".repeat(15));
