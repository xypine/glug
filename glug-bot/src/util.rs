pub fn format_with_spaces(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ' ');
        }
        result.insert(0, ch);
    }
    result
}

pub fn progress_bar(current: usize, total: usize) -> String {
    let filled_count = ((current * 10) / total).min(10);
    let empty_count = 10 - filled_count;

    let filled = "🍺".repeat(filled_count);
    let empty = "⏲".repeat(empty_count);

    format!("{}{}", filled, empty)
}
