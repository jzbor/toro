use std::fmt::Display;
use std::io::{Read, Write};
use std::process::Stdio;
use std::sync::{LazyLock, Mutex};
use std::{env, iter, process};

use colored::{Color, Colorize};
use regex::Regex;

use crate::config::{ColumnSelector, Config, ViewConfig};
use crate::error::{self, ToroError, ToroResult};
use crate::filter::Filter;
use crate::todotxt::tasks::*;
use crate::todotxt::file::*;

pub const COMPLETED_COLOR: Color = Color::BrightCyan;
pub const COMPLETION_DATE_COLOR: Color = Color::Cyan;
pub const CREATION_DATE_COLOR: Color = Color::Blue;
pub const DESCRIPTION_COLOR: Color = SELECTION_COLOR;
pub const DUE_COLOR: Color = Color::Green;
pub const OVERDUE_COLOR: Color = Color::Red;
pub const PENDING_COLOR: Color = Color::BrightBlue;
pub const PRIORITY_A_COLOR: Color = Color::BrightMagenta;
pub const PRIORITY_B_COLOR: Color = Color::BrightYellow;
pub const PRIORITY_COLOR: Color = PRIORITY_A_COLOR;
pub const SCHEDULED_COLOR: Color = Color::BrightGreen;
pub const SELECTION_COLOR: Color = Color::Yellow;


static READLINE: LazyLock<Mutex<rustyline::DefaultEditor>> = LazyLock::new(|| {
    error::resolve(rustyline::DefaultEditor::new()).into()
});


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

pub fn print_header(s: &str, level: usize) {
    let line = format!("{:#>level$} {}", "", s, level=level);
    println!("{}", line.yellow());
}

pub fn read_input(prompt: &str) -> ToroResult<String> {
    let mut rl = READLINE.lock().unwrap();

    match rl.readline(&prompt.bright_blue().bold().to_string()) {
        Ok(answer) => Ok(answer),
        Err(rustyline::error::ReadlineError::Eof) => Err(ToroError::EofError()),
        Err(e) => Err(e.into()),
    }
}

pub fn print_markdown(s: &str) {
    for line in s.lines() {
        let mut formatted = line.to_owned();
        let re_bold = Regex::new(r#"(?<b>\*\*.*?\*\*)"#).unwrap();
        let re_italic = Regex::new(r#"(?<b>\_.*?\_)"#).unwrap();

        // Add bold styling
        formatted = re_bold.replace_all(&formatted, |caps: &regex::Captures| {
            format!("{}", caps[0].to_string().bold())
        }).to_string();

        // Add italic styling
        formatted = re_italic.replace_all(&formatted, |caps: &regex::Captures| {
            format!("{}", caps[0].to_string().italic())
        }).to_string();

        if line.starts_with('#') {
            formatted = format!("{}", formatted.yellow().bold())
        }

        println!("{}", formatted);
    }
}

pub fn select_tasks_mut<'a>(file: &'a mut TodoTxtFile, config: &Config, filter_opt: Option<&Filter>, prompt: &str)
        -> ToroResult<(bool, Vec<&'a mut TodoTxtTask>)> {
    let tasks: Vec<_> = file.filtered_sorted_mut(filter_opt, &config.view.sort).collect();

    if config.view.auto_select && tasks.len() == 1 {
        println!("\n  [Auto selecting task]");
        return Ok((true, vec!(tasks.into_iter().next().unwrap())))
    }

    if config.view.fzf {
        let preview_cmd = format!("{}; CLICOLOR_FORCE=1 {} project --task \"$(echo \"{{}}\" | cut -d: -f2-)\"",
            config.view.cal_command.as_ref().map(|s| s.as_str()).unwrap_or("true"),
            env::current_exe()?.to_string_lossy());
        Ok((false, fzf_select(tasks, Some(prompt), Some(&preview_cmd))?))
    } else {
        let borrowed: Vec<_> = tasks.iter().map(|s| &**s).rev().collect();
        let numbering: Vec<_> = (0..tasks.len()).rev().collect();
        list_tasks(&borrowed, Some(numbering.as_slice()), config.columns, &config.view);
        println!();
        Ok((false, native_select(tasks, Some(prompt))?))
    }
}

pub fn native_select<T: Display>(items: impl IntoIterator<Item = T>, prompt: Option<&str>) -> ToroResult<Vec<T>> {
    let items: Vec<_> = items.into_iter().collect();
    let prompt = prompt.unwrap_or("> ");

    loop {
        let answer = read_input(prompt)?;
        let indices_result = answer.split(" ")
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

        let indices = match indices_result {
            Ok(nrs) => nrs,
            Err(_) => { eprintln!("{}", format!("Invalid input \"{}\"", answer).red()); continue },
        };

        if let Some(nr) = indices.iter().find(|n| **n >= items.len()) {
            eprintln!("{}", format!("Out of range: {}", nr + 1).red());
            continue
        }

        let selected: Vec<_> = items.into_iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, e)| e)
            .collect();

        return Ok(selected)
    }

}

pub fn fzf_select<T: Display>(items: impl IntoIterator<Item = T>, prompt: Option<&str>, preview: Option<&str>)
        -> ToroResult<Vec<T>> {
    let items: Vec<_> = items.into_iter().collect();
    let mut cmd = process::Command::new("fzf");

    cmd.args(["-d", ":", "--with-nth", "2..", "--multi"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if let Some(prompt) = prompt {
        cmd.arg("--prompt");
        cmd.arg(prompt);
    }

    if let Some(preview) = preview {
        cmd.arg("--preview");
        cmd.arg(preview);
    }

    let mut proc = cmd.spawn()?;

    let list = items.iter()
        .enumerate()
        .map(|(i, e)| format!("{}:{}", i, e))
        .collect::<Vec<_>>()
        .join("\n");

    let mut stdin = proc.stdin.take().unwrap();
    let mut stdout = proc.stdout.take().unwrap();
    stdin.write_all(list.as_bytes())?;

    let mut output = String::new();
    stdout.read_to_string(&mut output)?;

    let indices: Vec<usize> = output.lines()
        .map(|l| l.split(':').next().ok_or(ToroError::InvalidFzfResponse()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|n| n.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let selected: Vec<_> = items.into_iter()
        .enumerate()
        .filter(|(i, _)| indices.contains(i))
        .map(|(_, e)| e)
        .collect();

    if selected.is_empty() {
        return Err(ToroError::EofError())
    }

    proc.wait()?;

    Ok(selected)
}

pub fn select_field() -> ToroResult<FieldSelection> {
    loop {
        use FieldSelection::*;
        let fields = [ Completed, Priority, CompletionDate, CreationDate, Description, Due, Scheduled, ];
        let fields_label = fields.iter()
            .map(|f| f.to_string_fancy())
            .collect::<Vec<_>>()
            .join(", ");
        println!("Available fields: {}", fields_label);
        let answer = read_input("Please select a field: ")?;

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

pub fn list_tasks(tasks: &[&TodoTxtTask], numbering: Option<&[usize]>, columns: ColumnSelector, view: &ViewConfig) {
    if let Some(numbering) = numbering {
        let max_width = tasks.len().to_string().len();
        for (task, i) in tasks.iter().zip(numbering) {
            println!("{} {}", format!("{: >width$}:", i + 1, width = max_width).color(SELECTION_COLOR),
            task.to_string_fancy(columns, view));
        }
    } else {
        for task in tasks {
            println!("{}", task.to_string_fancy(columns, view));
        }
    }
}
