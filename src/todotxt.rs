use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::hash::Hash;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use chrono::{Datelike, NaiveDate, Utc};
use colored::Colorize;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;


use crate::error::{ToroError, ToroResult};
use crate::filter::{ColumnSelector, Filter};
use crate::exec::*;
use crate::interaction::*;

#[derive(Debug)]
pub struct TodoTxtFile {
    location: PathBuf,
    tasks: Vec<TodoTxtTask>,
}

#[derive(Hash, Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum DateRecord {
    Created(NaiveDate),
    CompletedCreated(NaiveDate, NaiveDate),
    NoDate
}

#[derive(Hash, Debug, Clone)]
pub enum DescriptionToken {
    Project(String),
    Context(String),
    Other(String),
    Meta(String, String),
}

#[derive(Debug)]
pub struct TodoTxtTask {
    completed: bool,
    priority: Option<char>,
    dates: DateRecord,
    description: Vec<DescriptionToken>,
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct TodoTxtParser;


impl TodoTxtFile {
    pub fn new(path: PathBuf) -> Self {
        TodoTxtFile {
            location: path,
            tasks: Vec::new(),
        }
    }

    pub fn load(path: PathBuf) -> ToroResult<Self> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ToroError::NamedIOError(path.clone(), e))?;
        let tasks = content.lines()
            .map(TodoTxtTask::parse)
            .collect::<ToroResult<Vec<_>>>()?;

        let mut file = TodoTxtFile {
            location: path,
            tasks,
        };

        file.resort();
        Ok(file)
    }

    pub fn store(&mut self) -> ToroResult<()> {
        self.resort();
        let mut file = fs::File::create(&self.location)?;
        let content = self.iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn git(&self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> ToroResult<()> {
        let orig: Vec<_> = args.into_iter().collect();
        let mut args: VecDeque<_> = orig.iter().map(|a| a.as_ref()).collect();
        let local = PathBuf::from(".");
        let path = self.location.parent().unwrap_or(&local);
        args.push_front(path.as_ref());
        args.push_front("-C".as_ref());
        exec("git", args)
    }

    pub fn dirty(&self) -> bool {
        self.git(["diff-index", "--quiet", "HEAD"])
            .is_err_and(|e| matches!(e, ToroError::ExternalCommandFailed(_)))
    }

    pub fn commit(&self, msg: &str) -> ToroResult<()> {
        if self.dirty() {
            eprintln!("\nCommitting changes...");
            let full_msg = format!("[toro] {}", msg);
            self.git(["commit", "-am", &full_msg])?;
        } else {
            eprintln!("\nNothing to commit.");
        }
        Ok(())

    }

    pub fn sync(&self) -> ToroResult<()> {
        eprintln!("\nSyncing git repo");
        self.git(["pull", "--rebase"])?;
        self.git(["push"])?;
        Ok(())
    }

    pub fn resort(&mut self) {
        self.tasks.sort_by_key(|t| Reverse(t.when_created().unwrap_or_default()));
        self.tasks.sort_by_key(|t| t.priority().unwrap_or('['));
        self.tasks.sort_by_key(|t| t.completed());
    }

    pub fn location(&self) -> &PathBuf {
        &self.location
    }

    pub fn filtered_tasks(&self, filter: &Filter) -> Vec<&TodoTxtTask> {
        self.tasks.iter()
            .filter(|t| filter.approves(t))
            .collect()
    }

    pub fn filtered_tasks_mut(&mut self, filter: &Filter) -> Vec<&mut TodoTxtTask> {
        self.tasks.iter_mut()
            .filter(|t| filter.approves(t))
            .collect()
    }

    pub fn list(&self, numbered: bool, reverse: bool, columns: ColumnSelector, filter_opt: Option<&Filter>) -> usize {
        let tasks = if let Some(filter) = filter_opt {
            self.filtered_tasks(filter)
        } else {
            self.tasks.iter().collect()
        };

        let ntasks = tasks.len();
        let enumerated = if reverse {
            Box::new(tasks.into_iter().enumerate().rev()) as Box<dyn Iterator<Item = (usize, &TodoTxtTask)>>
        } else {
            Box::new(tasks.into_iter().enumerate()) as Box<dyn Iterator<Item = (usize, &TodoTxtTask)>>
        };

        if numbered {
            let max_width = ntasks.to_string().len();
            for (i, task) in enumerated {
                println!("{} {}", format!("[{: >width$}]", i + 1, width = max_width).color(SELECTION_COLOR),
                    task.to_string_fancy(columns));
            }
        } else {
            for (_, task) in enumerated {
                println!("{}", task.to_string_fancy(columns));
            }
        }

        ntasks
    }
}

impl TodoTxtTask {
    pub fn parse(line: &str) -> ToroResult<Self> {
        let parsed = TodoTxtParser::parse(Rule::full_task, line)
            .map_err(|e| ToroError::SyntaxError(Box::new(e)))?
            .next().unwrap()
            .into_inner().next().unwrap();
        // println!("{:#?}", parsed);

        assert!(parsed.as_rule() == Rule::task);

        let mut inner = parsed.into_inner();
        let next_rule_is = |i: &Pairs<Rule>, r: Rule| i.peek().map(|p| p.as_rule()) == Some(r);

        let completed = if next_rule_is(&inner, Rule::completion_mark) {
            inner.next();
            true
        } else {
            false
        };

        let priority = if next_rule_is(&inner, Rule::priority) {
            let c = inner.next().unwrap()
                .as_str().chars().nth(1)
                .unwrap();
            Some(c)
        } else {
            None
        };

        let dates = if next_rule_is(&inner, Rule::completed_date) {
            let completed_pair = inner.next().unwrap()
                .into_inner()
                .next().unwrap();
            assert!(next_rule_is(&inner, Rule::created_date));
            let created_pair = inner.next().unwrap()
                .into_inner()
                .next().unwrap();
            DateRecord::CompletedCreated(parse_date(completed_pair.as_str())?, parse_date(created_pair.as_str())?)
        } else if next_rule_is(&inner, Rule::created_date) {
            let created_pair = inner.next().unwrap()
                .into_inner()
                .next().unwrap();
            DateRecord::Created(parse_date(created_pair.as_str())?)
        } else {
            DateRecord::NoDate
        };

        assert!(next_rule_is(&inner, Rule::description));
        let description_pair = inner.next().unwrap();
        let description = Self::parse_description(description_pair);

        Ok(TodoTxtTask { completed, priority, dates, description })
    }

    fn parse_description(pair: Pair<Rule>) -> Vec<DescriptionToken> {
        let description_inner = pair.into_inner();
        let mut description = Vec::new();

        for pair in description_inner {
            if pair.as_rule() == Rule::other {
                let val = pair.as_str().to_owned();
                description.push(DescriptionToken::Other(val));
            } else if pair.as_rule() == Rule::project {
                let val = pair.into_inner().next().unwrap().as_str().to_owned();
                description.push(DescriptionToken::Project(val));
            } else if pair.as_rule() == Rule::context {
                let val = pair.into_inner().next().unwrap().as_str().to_owned();
                description.push(DescriptionToken::Context(val));
            } else if pair.as_rule() == Rule::meta {
                let mut split = pair.as_str().split(":");
                let key = split.next().unwrap();
                let value = split.next().unwrap();
                description.push(DescriptionToken::Meta(key.to_owned(), value.to_owned()));
            } else {
                panic!("Unexpected pair ({:?})", pair.as_rule());
            }
        }

        description
    }

    pub fn to_string_fancy(&self, columns: ColumnSelector) -> String {
        let mut s = String::new();

        if columns.completed {
            if self.completed() {
                s.push_str("x ");
            } else {
                s.push_str("  ");
            }
        }

        if columns.priority {
            if let Some(prio) = self.priority {
                s.push_str(&format!("({}) ", prio));
            } else {
                s.push_str("    ");
            }
        }

        if columns.completion_date {
            let date_str = if let Some(date) = self.when_completed() {
                format!("{:0>4}-{:0>2}-{:0>2}", date.year(), date.month(), date.day())
            } else {
                String::new()
            };

            s.push_str(&format!("{:<10} ", date_str));
        }

        if columns.creation_date {
            let date_str = if let Some(date) = self.when_created() {
                format!("{:0>4}-{:0>2}-{:0>2}", date.year(), date.month(), date.day())
            } else {
                String::new()
            };

            s.push_str(&format!("{:<10} ", date_str));
        }

        s.push_str(&self.description_fancy());

        if self.completed() {
            s = s.strikethrough().to_string();
        } else if self.priority() == Some('A') {
            s = s.color(PRIORITY_A_COLOR).to_string();
        } else if self.priority() == Some('B') {
            s = s.color(PRIORITY_B_COLOR).to_string();
        }

        s
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn when_completed(&self) -> Option<NaiveDate> {
        self.dates.completed()
    }

    pub fn when_created(&self) -> Option<NaiveDate> {
        self.dates.created()
    }

    pub fn complete(&mut self) {
        self.completed = true;
        self.dates.set_completed(Utc::now().naive_local().into())
    }

    pub fn uncomplete(&mut self) {
        self.completed = false;
        self.dates.set_not_completed()
    }

    pub fn set_completed(&mut self, val: bool) {
        if val {
            self.complete();
        } else {
            self.uncomplete();
        }
    }

    pub fn set_completion_date(&mut self, date_opt: Option<NaiveDate>) {
        match date_opt {
            Some(date) => self.dates.set_completed(date),
            None => self.dates.set_not_completed(),
        }
    }

    pub fn set_creation_date(&mut self, date_opt: Option<NaiveDate>) {
        use DateRecord::*;
        match date_opt {
            Some(date) => self.dates.set_created(date),
            None => self.dates = match self.dates {
                Created(_) => NoDate,
                CompletedCreated(completed, _) => CompletedCreated(completed, completed),
                NoDate => NoDate,
            },
        }
    }

    pub fn priority(&self) -> Option<char> {
        self.priority
    }

    pub fn set_priority(&mut self, priority: Option<char>) {
        self.priority = priority;
    }

    pub fn description(&self) -> String {
        self.description.iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn description_fancy(&self) -> String {
        let mut s = String::new();

        for token in &self.description {
            s.push_str(&token.to_string_colored());
        }

        s
    }

    pub fn set_description(&mut self, description: &str) -> ToroResult<()> {
        let parsed = TodoTxtParser::parse(Rule::full_description, description)
            .map_err(|e| ToroError::SyntaxError(Box::new(e)))?
            .next().unwrap()
            .into_inner().next().unwrap();
        self.description = Self::parse_description(parsed);
        Ok(())
    }
}

impl DateRecord {
    pub fn created(&self) -> Option<NaiveDate> {
        use DateRecord::*;
        match *self {
            Created(date) => Some(date),
            CompletedCreated(_, date) => Some(date),
            NoDate => None,
        }
    }

    pub fn completed(&self) -> Option<NaiveDate> {
        use DateRecord::*;
        match *self {
            CompletedCreated(date, _) => Some(date),
            _ => None,
        }
    }

    pub fn with_created(self, created: NaiveDate) -> Self {
        use DateRecord::*;
        match self {
            CompletedCreated(comp, _) => CompletedCreated(comp, created),
            _ => Created(created),
        }
    }

    pub fn with_completed(self, completed: NaiveDate) -> Self {
        use DateRecord::*;
        match self {
            CompletedCreated(_, crea) => CompletedCreated(completed, crea),
            Created(crea) => CompletedCreated(completed, crea),
            NoDate => CompletedCreated(completed, completed)
        }
    }

    pub fn without_completed(self) -> Self {
        use DateRecord::*;
        match self {
            CompletedCreated(_, crea) => Created(crea),
            other => other,
        }
    }

    pub fn set_created(&mut self, created: NaiveDate) {
        *self = self.with_created(created)
    }

    pub fn set_completed(&mut self, completed: NaiveDate) {
        *self = self.with_completed(completed)
    }

    pub fn set_not_completed(&mut self) {
        *self = self.without_completed()
    }
}

impl DescriptionToken {
    fn to_string_colored(&self) -> String {
        let plain_string = self.to_string();
        use DescriptionToken::*;
        match self {
            Project(_) => plain_string.bold().to_string(),
            Context(_) => plain_string.italic().to_string(),
            Meta(..) => plain_string.dimmed().to_string(),
            Other(_) => plain_string,
        }
    }
}


impl Deref for TodoTxtFile {
    type Target = Vec<TodoTxtTask>;

    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

impl DerefMut for TodoTxtFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tasks
    }
}

impl Display for DescriptionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DescriptionToken::*;
        match self {
            Project(s) => write!(f, "+{}", s),
            Context(s) => write!(f, "@{}", s),
            Meta(k, v) => write!(f, "{}:{}", k, v),
            Other(s) => s.fmt(f),
        }
    }
}

impl Display for DateRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DateRecord::*;
        match self {
            Created(created) => write!(f, "{:0>4}-{:0>2}-{:0>2} ", created.year(), created.month(), created.day()),
            CompletedCreated(completed, created) => write!(f, "{:0>4}-{:0>2}-{:0>2} {:0>4}-{:0>2}-{:0>2} ",
                completed.year(), completed.month(), completed.day(),
                created.year(), created.month(), created.day()),
            NoDate => Ok(()),
        }
    }
}

impl Display for TodoTxtTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.completed() {
            write!(f, "x ")?;
        }

        if let Some(prio) = self.priority {
            write!(f, "({}) ", prio)?;
        }

        write!(f, "{} ", self.dates)?;

        for token in &self.description {
            token.fmt(f)?;
        }

        Ok(())
    }
}

impl Hash for TodoTxtTask {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.completed.hash(state);
        self.priority.hash(state);
        self.dates.hash(state);
        self.description.hash(state);
    }
}

pub fn parse_date(input: &str) -> ToroResult<NaiveDate> {
    let mut parts = input.splitn(3, "-");

    let year_str = parts.next().ok_or_else(|| ToroError::DateInputError(input.to_owned()))?;
    let month_str = parts.next().ok_or_else(|| ToroError::DateInputError(input.to_owned()))?;
    let day_str = parts.next().ok_or_else(|| ToroError::DateInputError(input.to_owned()))?;

    let year = str::parse(year_str).map_err(|_| ToroError::DateInputError(input.to_owned()))?;
    let month = str::parse(month_str).map_err(|_| ToroError::DateInputError(input.to_owned()))?;
    let day = str::parse(day_str).map_err(|_| ToroError::DateInputError(input.to_owned()))?;

    NaiveDate::default()
        .with_year(year)
        .ok_or_else(|| ToroError::DateInputError(input.to_owned()))?
        .with_month(month)
        .ok_or_else(|| ToroError::DateInputError(input.to_owned()))?
        .with_day(day)
        .ok_or_else(|| ToroError::DateInputError(input.to_owned()))
}
