use std::io::Write;

use colored::{Color, Colorize};

use crate::filter::{ColumnSelector, Filter};
use crate::todotxt::TodoTxtFile;

pub const COMPLETED_COLOR: Color = Color::BrightCyan;
pub const PENDING_COLOR: Color = Color::BrightBlue;
pub const SELECTION_COLOR: Color = Color::Yellow;
pub const PRIORITY_A_COLOR: Color = Color::BrightMagenta;
pub const PRIORITY_B_COLOR: Color = Color::BrightYellow;


pub fn announce(s: &str) {
    println!("\n{}", format!("=> {s}").green());
}

pub fn select_tasks(file: &TodoTxtFile, columns: ColumnSelector, filter_opt: Option<&Filter>) -> Vec<usize> {
    let ntasks = file.list(true, true, columns, filter_opt);
    println!();

    loop {

        let answer = match ask("Please select one or multiple tasks:") {
            Some(answer) => answer,
            None => return Vec::new(),
        };
        let numbers_result = answer.split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| str::parse::<usize>(s).map(|i| i - 1))
            .collect::<Result<Vec<_>, _>>();

        let nrs = match numbers_result {
            Ok(nrs) => nrs,
            Err(_) => { eprintln!("{}", format!("Invalid input \"{}\"", answer).red()); continue },
        };

        match nrs.iter().find(|n| **n >= ntasks) {
            None => return nrs,
            Some(nr) => { eprintln!("{}", format!("Out of range: {}", nr + 1).red()); continue },
        }
    }
}

pub fn ask(question: &str) -> Option<String> {
    loop {
        print!("{question} ");
        let _ = std::io::stdout().flush();

        let mut answer = String::new();
        match std::io::stdin().read_line(&mut answer) {
            Ok(_) => (),
            Err(_) => continue,
        };

        match answer.as_str() {
            "" => return None,
            "\n" | "\r\n" => continue,
            other => return Some(other.trim().to_owned()),
        }
    }
}
