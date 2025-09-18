use rgrep::run;
use std::fs;
use std::io::{self, BufRead, IsTerminal};

fn main() -> io::Result<()> {
    let filename = std::env::args().nth(1);

    let all_lines = match filename {
        Some(path) => {
            let contents = fs::read_to_string(path)?;
            Ok(contents.lines().map(|s| s.to_string()).collect())
        }
        None => {
            if io::stdin().is_terminal() {
                eprintln!("Error: No input provided.");
                std::process::exit(1);
            } else {
                let stdin = io::stdin();
                let lines: Result<Vec<String>, _> = stdin.lock().lines().collect();
                lines
            }
        }
    };

    match all_lines {
        Ok(lines) => run(lines),
        Err(e) => Err(e),
    }
}

// fn is_interactive() -> bool {
//     std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
// }
