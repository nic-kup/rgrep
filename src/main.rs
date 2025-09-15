use crossterm::{
    cursor::{MoveDown, MoveTo, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{Clear, ClearType, size},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    // unsafe {
    //     std::env::set_var("RUST_BACKTRACE", "1");
    // }
    let (_width, height) = size()?;

    let filename = std::env::args().nth(1).expect("Missing <filename>");
    execute!(io::stdout(), MoveToColumn(0))?;

    let contents = fs::read_to_string(filename)?;
    let all_lines: Vec<&str> = contents.lines().collect();

    enable_raw_mode()?;

    let mut query = String::new();
    let mut query_change = false;
    let ansi_color_len = "\x1b[1;31m\x1b[0m".len();

    let mut matches: Vec<(usize, (String, u32))> = all_lines
        .iter()
        .enumerate()
        .map(|(num, line)| (num, fuzzy_match(&query, line)))
        .filter_map(|(num, opt)| opt.map(|val| (num, val)))
        .take((height as usize).saturating_sub(5))
        .collect();

    'main_loop: loop {
        let (width, height) = size()?;
        // clear and reset cursor
        // TODO move cursor to after query
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        print!("{}", query);
        execute!(io::stdout(), MoveToColumn(0), MoveDown(1))?;
        println!("Press 'Esc' to quit | {} | {}", width, ansi_color_len);
        execute!(io::stdout(), MoveToColumn(0))?;
        println!("{}", "-".repeat(width as usize));
        execute!(io::stdout(), MoveToColumn(0))?;

        if query_change {
            matches = all_lines
                .iter()
                .enumerate()
                .map(|(num, line)| (num, fuzzy_match(&query, line)))
                .filter_map(|(num, opt)| opt.map(|val| (num, val)))
                .take((height as usize).saturating_sub(5))
                .collect();

            query_change = false;
        }

        matches.sort_by_key(|(_num, val)| val.1);
        let max_line_len: usize = matches
            .iter()
            .map(|f| f.1.0.len())
            .max()
            .map_or(String::new(), |n| n.to_string())
            .len();

        for (num, (line, _score)) in matches.iter() {
            let display_line =
                if line.len() - ansi_color_len * query.len() > (width as usize).saturating_sub(3) {
                    line.chars()
                        .take((width as usize).saturating_sub(5))
                        .collect::<String>()
                        + "..."
                        + "\x1b[0m"
                } else {
                    line.to_string() + "\x1b[0m"
                };
            println!(
                "{:>width$}: {}",
                num + 1,
                display_line,
                width = max_line_len
            );
            execute!(io::stdout(), MoveToColumn(0))?;
        }

        if matches.is_empty() && !query.is_empty() {
            println!("No matches found");
            execute!(io::stdout(), MoveToColumn(0))?;
        }

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Esc => {
                    break 'main_loop;
                }
                KeyCode::Char(c) => {
                    query_change = true;
                    query.push(c);
                }
                KeyCode::Backspace => {
                    query_change = true;
                    query.pop();
                }
                _ => {}
            }
        }
    }

    execute!(io::stdout(), MoveToColumn(0))?;
    disable_raw_mode()?;
    Ok(())
}

/// Fuzzy match run on a per line basis
fn fuzzy_match(pattern: &str, text: &str) -> Option<(String, u32)> {
    if pattern.is_empty() {
        return Some((text.to_string(), 0));
    }

    if text.is_empty() {
        return None;
    }

    let positions: Vec<usize> = text
        .chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if c == pattern.chars().next().expect("Already checked empty!") {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let pattern_chars: Vec<char> = pattern.chars().collect();

    let mut best_score = u32::MAX;
    let mut optimal_result = String::new();

    for pos in positions {
        let mut first_match: bool = false;

        let mut result = String::new();
        result.push_str(&text[..pos]);

        let mut score: u32 = 0;
        let mut pattern_index = 0;

        for text_char in text[pos..].chars() {
            if pattern_chars.len() > pattern_index
                && text_char
                    .to_lowercase()
                    .eq(pattern_chars[pattern_index].to_lowercase())
            {
                result.push_str("\x1b[1;31m");
                result.push(text_char);
                result.push_str("\x1b[0m");

                first_match = true;
                pattern_index += 1;
            } else {
                result.push(text_char);
                if first_match && pattern_chars.len() > pattern_index {
                    score += 1;
                }
            }
        }

        if score < best_score && pattern_index == pattern_chars.len() {
            best_score = score;
            optimal_result = result;
        }
    }

    if best_score == u32::MAX {
        return None;
    }

    Some((optimal_result, best_score))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_matching() {
        let result = fuzzy_match("abc", "aXbXXcXXabcXX");
        assert!(result.is_some());
        let (_highlighted, score) = result.unwrap();
        assert_eq!(score, 0);
    }
}
