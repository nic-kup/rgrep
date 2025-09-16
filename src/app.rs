use crate::fuzzy_match;
use crossterm::{
    cursor::{MoveDown, MoveTo, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{Clear, ClearType, size},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

pub fn run(all_lines: Vec<String>) -> std::io::Result<()> {
    let (_width, height) = size()?;

    execute!(io::stderr(), MoveToColumn(0))?;

    enable_raw_mode()?;

    let mut query = String::new();
    let mut query_change = false;
    let ansi_color_len = "\x1b[1;31m\x1b[0m".len();

    let mut selected_index: usize = 0;

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
        execute!(io::stderr(), Clear(ClearType::All), MoveTo(0, 0))?;

        eprint!("{}", query);
        execute!(io::stderr(), MoveToColumn(0), MoveDown(1))?;
        eprintln!(
            "Press 'Esc' to quit | {} | {}",
            width, ansi_color_len
        );
        execute!(io::stderr(), MoveToColumn(0))?;
        eprintln!("{}", "-".repeat(width as usize));
        execute!(io::stderr(), MoveToColumn(0))?;

        if query_change {
            matches = all_lines
                .iter()
                .enumerate()
                .map(|(num, line)| (num, fuzzy_match(&query, line)))
                .filter_map(|(num, opt)| opt.map(|val| (num, val)))
                .take((height as usize).saturating_sub(5))
                .collect();

            query_change = false;
            selected_index = 0;
        }

        matches.sort_by_key(|(_num, val)| val.1);
        let max_line_len: usize = matches
            .iter()
            .map(|f| f.1.0.len())
            .max()
            .map_or(String::new(), |n| n.to_string())
            .len();

        for (i, (num, (line, _score))) in matches.iter().enumerate() {
            let prefix =
                if i == selected_index { "-> " } else { "   " };
            let display_line = if line.len()
                - ansi_color_len * query.len()
                > (width as usize).saturating_sub(3)
            {
                line.chars()
                    .take((width as usize).saturating_sub(5))
                    .collect::<String>()
                    + "..."
                    + "\x1b[0m"
            } else {
                line.to_string() + "\x1b[0m"
            };
            eprintln!(
                "{}{:>width$}: {}",
                prefix,
                num + 1,
                display_line,
                width = max_line_len
            );
            execute!(io::stderr(), MoveToColumn(0))?;
        }

        if matches.is_empty() && !query.is_empty() {
            eprintln!("No matches found");
            execute!(io::stderr(), MoveToColumn(0))?;
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
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index =
                            selected_index.saturating_sub(1);
                    }
                }
                KeyCode::Down => {
                    if selected_index + 1 < matches.len() {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    if !matches.is_empty() {
                        let (_line_num, (line, _score)) =
                            &matches[selected_index];
                        println!("{}", line);
                        break 'main_loop;
                    }
                }
                _ => {}
            }
        }
    }

    execute!(io::stderr(), MoveToColumn(0))?;
    disable_raw_mode()?;
    Ok(())
}
