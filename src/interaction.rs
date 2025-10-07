use std::fmt::Display;
use std::io::Write;

use colored::{Color, Colorize};

use crate::filter::{ColumnSelector, Filter};
use crate::todotxt::TodoTxtFile;

pub const COMPLETED_COLOR: Color = Color::BrightCyan;
pub const PENDING_COLOR: Color = Color::BrightBlue;
pub const SELECTION_COLOR: Color = Color::Yellow;
pub const PRIORITY_A_COLOR: Color = Color::BrightMagenta;
pub const PRIORITY_B_COLOR: Color = Color::BrightYellow;
pub const PRIORITY_COLOR: Color = PRIORITY_A_COLOR;
pub const COMPLETION_DATE_COLOR: Color = Color::Cyan;
pub const CREATION_DATE_COLOR: Color = Color::Blue;
pub const DESCRIPTION_COLOR: Color = SELECTION_COLOR;


#[derive(Copy, Clone, Debug)]
pub enum FieldSelection {
    Completed,
    Priority,
    CompletionDate,
    CreationDate,
    Description,
}


impl FieldSelection {
    pub fn to_string_fancy(self) -> String {
        use FieldSelection::*;
        let color = match self {
            Completed => COMPLETED_COLOR,
            Priority => PRIORITY_COLOR,
            CompletionDate => COMPLETION_DATE_COLOR,
            CreationDate => CREATION_DATE_COLOR,
            Description => DESCRIPTION_COLOR,
        };
        self.to_string().color(color).to_string()
    }
}


impl Display for FieldSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FieldSelection::*;
        match self {
            Completed => write!(f, "completed"),
            Priority => write!(f, "priority"),
            CompletionDate => write!(f, "completion-date"),
            CreationDate => write!(f, "creation-date"),
            Description => write!(f, "description"),
        }
    }
}


pub fn announce(s: &str) {
    println!("\n{}\n", format!("=> {s}").green());
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

pub fn select_field() -> Option<FieldSelection> {
    loop {
        use FieldSelection::*;
        let fields = [ Completed, Priority, CompletionDate, CreationDate, Description ];
        let fields_label = fields.iter()
            .map(|f| f.to_string_fancy())
            .collect::<Vec<_>>()
            .join(", ");
        let answer = match ask(&format!("Available fields: {}\nPlease select a field:", fields_label)) {
            Some(answer) => answer,
            None => return None,
        };

        for field in fields {
            if field.to_string().starts_with(&answer) {
                return Some(field);
            }
        }

        eprintln!("{}", format!("Invalid field: {}", answer).red());
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
