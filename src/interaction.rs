use std::fmt::Display;
use std::iter;

use colored::{Color, Colorize};

use crate::config::Config;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::todotxt::{TodoTxtFile, TodoTxtTask};

pub const COMPLETED_COLOR: Color = Color::BrightCyan;
pub const COMPLETION_DATE_COLOR: Color = Color::Cyan;
pub const CREATION_DATE_COLOR: Color = Color::Blue;
pub const DESCRIPTION_COLOR: Color = SELECTION_COLOR;
pub const DUE_COLOR: Color = Color::BrightCyan;
pub const PENDING_COLOR: Color = Color::BrightBlue;
pub const PRIORITY_A_COLOR: Color = Color::BrightMagenta;
pub const PRIORITY_B_COLOR: Color = Color::BrightYellow;
pub const PRIORITY_COLOR: Color = PRIORITY_A_COLOR;
pub const SCHEDULED_COLOR: Color = Color::BrightBlue;
pub const SELECTION_COLOR: Color = Color::Yellow;


#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum FieldSelection {
    Completed,
    CompletionDate,
    CreationDate,
    Description,
    Due,
    Priority,
    Scheduled,
}


impl FieldSelection {
    pub fn to_string_fancy(self) -> String {
        self.to_string().color(self.color()).to_string()
    }

    pub fn color(self) -> Color {
        use FieldSelection::*;
        match self {
            Completed => COMPLETED_COLOR,
            CompletionDate => COMPLETION_DATE_COLOR,
            CreationDate => CREATION_DATE_COLOR,
            Description => DESCRIPTION_COLOR,
            Due => DUE_COLOR,
            Priority => PRIORITY_COLOR,
            Scheduled => SCHEDULED_COLOR,
        }
    }
}

impl Default for FieldSelection {
    fn default() -> Self {
        FieldSelection::Description
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
            Due => write!(f, "due"),
            Scheduled => write!(f, "scheduled"),
        }
    }
}

pub fn announce(s: &str) {
    println!("\n{}\n", format!("=> {s}").green());
}

pub fn select_tasks_mut<'a>(rl: &mut rustyline::DefaultEditor, file: &'a mut TodoTxtFile, config: &Config, filter_opt: Option<&Filter>)
        -> ToroResult<(bool, Vec<&'a mut TodoTxtTask>)> {
    let ntasks = file.list(true, true, config.columns, &config.view, filter_opt);

    if config.view.auto_select && ntasks == 1 {
        println!("\n  [Auto selecting task]");
        let mut tasks = file.filtered_sorted_mut(filter_opt, &config.view.sort);
        return Ok((true, vec!(tasks.next().unwrap())))
    }

    println!();

    loop {
        let answer = match rl.readline("Please select one or multiple tasks: ") {
            Ok(answer) => answer,
            Err(rustyline::error::ReadlineError::Eof) => return Err(ToroError::EofError()),
            Err(e) => return Err(e.into()),
        };
        let numbers_result = answer.split(" ")
            .filter(|s| !s.is_empty())
            .flat_map(|s| {
                if let Some((s1, s2)) = s.split_once("-") {
                    let i1 = str::parse::<usize>(s1).map(|i| i - 1);
                    let i2 = str::parse::<usize>(s2).map(|i| i - 1);
                    match i1.and_then(|i1| i2.map(|i2| i1..=i2)) {
                        Ok(range) => Box::new(range.map(Ok)) as Box<dyn Iterator<Item = Result<usize, _>>>,
                        Err(e) => Box::new(iter::once(Err(e))) as Box<dyn Iterator<Item = Result<usize, _>>>,
                    }
                } else {
                    Box::new(iter::once(str::parse::<usize>(s).map(|i| i - 1))) as Box<dyn Iterator<Item = Result<usize, _>>>
                }
            })
            .collect::<Result<Vec<_>, _>>();

        let nrs = match numbers_result {
            Ok(nrs) => nrs,
            Err(_) => { eprintln!("{}", format!("Invalid input \"{}\"", answer).red()); continue },
        };

        if let Some(nr) = nrs.iter().find(|n| **n >= ntasks) {
            eprintln!("{}", format!("Out of range: {}", nr + 1).red());
            continue
        }

        let selected: Vec<_> = file.filtered_sorted_mut(filter_opt, &config.view.sort)
            .enumerate()
            .filter_map(|(i, t)| if nrs.contains(&i) { Some(t) } else { None })
            .collect();
        return Ok((false, selected))
    }
}

pub fn select_field(rl: &mut rustyline::DefaultEditor) -> ToroResult<FieldSelection> {
    loop {
        use FieldSelection::*;
        let fields = [ Completed, Priority, CompletionDate, CreationDate, Description, Due, Scheduled, ];
        let fields_label = fields.iter()
            .map(|f| f.to_string_fancy())
            .collect::<Vec<_>>()
            .join(", ");
        let answer = match rl.readline(&format!("Available fields: {}\nPlease select a field: ", fields_label)) {
            Ok(answer) => answer,
            Err(rustyline::error::ReadlineError::Eof) => return Err(ToroError::EofError()),
            Err(e) => return Err(e.into()),
        };

        let mut matches = fields.into_iter()
            .filter(|f| f.to_string().starts_with(&answer));

        if let Some(matching_field) = matches.next() {
            match matches.next() {
                Some(_other_field) => {
                    eprintln!("{}", format!("Ambiguous selection: {}", answer).red());
                    continue
                },
                None => return Ok(matching_field),
            }
        }

        for field in fields {
            if field.to_string().starts_with(&answer) {
                return Ok(field);
            }
        }

        eprintln!("{}", format!("Invalid field: {}", answer).red());
    }
}
