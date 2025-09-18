use crate::fuzzy_match;
use std::io::{self, IsTerminal, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, terminal_size};

pub fn run(all_lines: Vec<String>) -> std::io::Result<()> {
    let (_width, height) = terminal_size()?;

    let _stdout = if io::stdout().is_terminal() {
        Some(io::stdout().into_raw_mode()?)
    } else {
        None
    };

    #[cfg(unix)]
    let tty = std::fs::File::open("/dev/tty")?;

    #[cfg(windows)]
    let tty = std::fs::File::open("CONIN$")?;

    let mut keys = tty.keys();

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
        let (width, height) = terminal_size()?;
        //
        // clear and reset cursor
        // TODO move cursor to after query
        eprint!("{}{}", clear::All, cursor::Goto(1, 1));

        eprint!("{}", query);

        eprint!("{}", cursor::Goto(1, 2));
        eprintln!("Press 'Esc' to quit | {} | {}", width, ansi_color_len);

        eprint!("{}", cursor::Goto(1, 3));
        eprintln!("{}", "-".repeat(width as usize));

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
            eprint!("{}", cursor::Goto(1, (4 + i) as u16));

            let prefix = if i == selected_index { "-> " } else { "   " };

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
            eprintln!(
                "{}{:>width$}: {}",
                prefix,
                num + 1,
                display_line,
                width = max_line_len
            );
        }

        if matches.is_empty() && !query.is_empty() {
            eprint!("{}", cursor::Goto(1, 4));
            eprintln!("No matches found");
        }

        eprint!("{}", cursor::Goto((query.len() + 1) as u16, 1));
        io::stdout().flush()?;

        match keys.next() {
            Some(Ok(key)) => match key {
                Key::Esc => {
                    break 'main_loop;
                }
                Key::Char(c) => {
                    if c == '\n' {
                        if !matches.is_empty() {
                            let (_line_num, (line, _score)) = &matches[selected_index];
                            println!("{}{}", clear::All, cursor::Goto(1, 1));
                            println!("{}", line);
                            break 'main_loop;
                        }
                    } else {
                        query_change = true;
                        query.push(c);
                    }
                }
                Key::Backspace => {
                    query_change = true;
                    query.pop();
                }
                Key::Up => {
                    if selected_index > 0 {
                        selected_index = selected_index.saturating_sub(1);
                    }
                }
                Key::Down => {
                    if selected_index + 1 < matches.len() {
                        selected_index += 1;
                    }
                }
                _ => {}
            },
            Some(Err(e)) => return Err(e),
            None => {}
        }
    }

    println!("{}{}", clear::All, cursor::Goto(1, 1));
    Ok(())
}
