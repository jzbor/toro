use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::hash::Hash;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use chrono::{NaiveDate, TimeDelta, Utc};
use colored::Colorize;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;


use crate::config::{ColumnSelector, SortBy, ViewConfig};
use crate::date::*;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::exec::*;
use crate::interaction::*;


const DUE_KEY: &str = "due";
const SCHEDULED_KEY: &str = "scheduled";


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
        eprintln!("loaded {} entries", 1);
        let tasks = content.lines()
            .map(TodoTxtTask::parse)
            .collect::<ToroResult<Vec<_>>>()?;
        // eprintln!("result: {:#?}", res);
        // eprintln!("{}", res.map_err(|e| e.to_string()).unwrap_err());
        // todo!();
        // let tasks = res?;

        let file = TodoTxtFile {
            location: path,
            tasks,
        };

        Ok(file)
    }

    pub fn store(&mut self) -> ToroResult<()> {
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

    pub fn filtered_sorted(&self, filter_opt: Option<&Filter>, sort_by: &[SortBy]) -> impl Iterator<Item = &TodoTxtTask> {
        let mut entries: Vec<_> = if let Some(filter) = filter_opt {
            self.tasks.iter()
                .filter(|t| filter.approves(t))
                .collect()
        } else {
            self.tasks.iter()
                .collect()
        };

        let naive_now: NaiveDate = Utc::now().naive_local().into();

        for key in sort_by {
            use SortBy::*;
            match key {
                Description => entries.sort_by_key(|e| e.description()),
                Created => entries.sort_by_key(|e| Reverse(e.when_created().unwrap_or_default())),
                Completed => entries.sort_by_key(|e| e.completed()),
                Priority => entries.sort_by_key(|e| e.priority().unwrap_or('[')),
                Nop => (),
                Due => entries.sort_by_key(|e| e.when_due().unwrap_or_default().map(|d| d - naive_now).unwrap_or(TimeDelta::MAX)),
                Scheduled => entries.sort_by_key(|e| e.when_scheduled().unwrap_or_default().map(|d| d - naive_now).unwrap_or(TimeDelta::MAX)),
            }
        }

        entries.into_iter()
    }

    pub fn filtered_sorted_mut(&mut self, filter_opt: Option<&Filter>, sort_by: &[SortBy]) -> impl Iterator<Item = &mut TodoTxtTask> {
        let mut entries: Vec<_> = if let Some(filter) = filter_opt {
            self.tasks.iter_mut()
                .filter(|t| filter.approves(t))
                .collect()
        } else {
            self.tasks.iter_mut()
                .collect()
        };

        let naive_now: NaiveDate = Utc::now().naive_local().into();

        for key in sort_by {
            use SortBy::*;
            match key {
                Completed => entries.sort_by_key(|e| e.completed()),
                Created => entries.sort_by_key(|e| Reverse(e.when_created().unwrap_or_default())),
                Description => entries.sort_by_key(|e| e.description()),
                Due => entries.sort_by_key(|e| e.when_due().unwrap_or_default().map(|d| d - naive_now).unwrap_or(TimeDelta::MAX)),
                Nop => (),
                Priority => entries.sort_by_key(|e| e.priority().unwrap_or('[')),
                Scheduled => entries.sort_by_key(|e| e.when_scheduled().unwrap_or_default().map(|d| d - naive_now).unwrap_or(TimeDelta::MAX)),
            }
        }

        entries.into_iter()
    }

    pub fn location(&self) -> &PathBuf {
        &self.location
    }

    pub fn list(&self, numbered: bool, reverse: bool, columns: ColumnSelector, view: &ViewConfig, filter_opt: Option<&Filter>) -> usize {
        let tasks: Vec<_> = self.filtered_sorted(filter_opt, &view.sort).collect();

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
                    task.to_string_fancy(columns, view));
            }
        } else {
            for (_, task) in enumerated {
                println!("{}", task.to_string_fancy(columns, view));
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
            if pair.as_rule() == Rule::other || pair.as_rule() == Rule::other_whitespace {
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
                let key = split.next().unwrap().to_owned();
                let mut value = split.next().unwrap().to_owned();

                if key == DUE_KEY && let Ok(new) = parse_date(&value) {
                    value = format_date(new, false);
                }
                if key == SCHEDULED_KEY && let Ok(new) = parse_date(&value) {
                    value = format_date(new, false);
                }
                description.push(DescriptionToken::Meta(key, value));
            } else {
                panic!("Unexpected pair ({:?})", pair.as_rule());
            }
        }

        description
    }

    pub fn to_string_fancy(&self, columns: ColumnSelector, view: &ViewConfig) -> String {
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
                format_date(date, view.pretty_dates)
            } else {
                String::new()
            };

            s.push_str(&format!("{:<10} ", date_str));
        }

        if columns.creation_date {
            let date_str = if let Some(date) = self.when_created() {
                format_date(date, view.pretty_dates)
            } else {
                String::new()
            };

            s.push_str(&format!("{:<10} ", date_str));
        }

        s.push_str(&self.description_fancy(view));

        if self.completed() {
            s = s.strikethrough().to_string();
        } else if self.priority() == Some('A') {
            s = s.color(PRIORITY_A_COLOR).to_string();
        } else if self.priority() == Some('B') {
            s = s.color(PRIORITY_B_COLOR).to_string();
        }

        s
    }

    pub fn project(&self) -> Option<&str> {
        self.description.iter()
            .find_map(|t| if let DescriptionToken::Project(p) = t { Some(p.as_str()) } else { None })
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

    pub fn when_due(&self) -> ToroResult<Option<NaiveDate>> {
        match self.meta(DUE_KEY) {
            Some(s) => Ok(Some(parse_date(s)?)),
            None => Ok(None),
        }
    }

    pub fn when_scheduled(&self) -> ToroResult<Option<NaiveDate>> {
        match self.meta(SCHEDULED_KEY) {
            Some(s) => Ok(Some(parse_date(s)?)),
            None => Ok(None),
        }
    }

    pub fn set_due(&mut self, date_opt: Option<NaiveDate>) {
        if let Some(date) = date_opt {
            for token in self.description.iter_mut() {
                if let DescriptionToken::Meta(k, v) = token && k == DUE_KEY {
                    *v = format_date(date, false);
                    return
                }
            }
            self.description.push(DescriptionToken::Other(" ".to_owned()));
            self.description.push(DescriptionToken::Meta(DUE_KEY.to_owned(), format_date(date, false)));
        } else {
            self.description.retain(|e|
                if let DescriptionToken::Meta(k, _) = e {
                    k != DUE_KEY
                } else {
                    true
                }
            );
        }
    }

    pub fn set_scheduled(&mut self, date_opt: Option<NaiveDate>) {
        if let Some(date) = date_opt {
            for token in self.description.iter_mut() {
                if let DescriptionToken::Meta(k, v) = token && k == SCHEDULED_KEY {
                    *v = format_date(date, false);
                    return
                }
            }
            self.description.push(DescriptionToken::Other(" ".to_owned()));
            self.description.push(DescriptionToken::Meta(SCHEDULED_KEY.to_owned(), format_date(date, false)));
        } else {
            self.description.retain(|e|
                if let DescriptionToken::Meta(k, _) = e {
                    k != SCHEDULED_KEY
                } else {
                    true
                }
            );
        }
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

    pub fn description_fancy(&self, view: &ViewConfig) -> String {
        let mut s = String::new();

        for token in &self.description {
            if let DescriptionToken::Meta(k, v) = token
                    && (k == DUE_KEY || k == SCHEDULED_KEY)
                    && let Ok(date) = parse_date(v) {
                s.push_str(&token.colored(&format!("{}:{}", k, format_date(date, view.pretty_dates))));
            } else {
                s.push_str(&token.to_string_colored());
            }
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

    pub fn meta(&self, key: &str) -> Option<&str> {
        for token in &self.description {
            if let DescriptionToken::Meta(k, v) = token && k == key {
                return Some(v)
            }
        }

        None
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
    fn colored(&self, s: &str) -> String {
        use DescriptionToken::*;
        match self {
            Project(_) => s.bold().to_string(),
            Context(_) => s.italic().to_string(),
            Meta(..) => s.dimmed().to_string(),
            Other(_) => s.to_owned(),
        }
    }

    fn to_string_colored(&self) -> String {
        self.colored(&self.to_string())
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
            Created(created) => write!(f, "{} ", format_date(*created, false)),
            CompletedCreated(completed, created) => write!(f, "{} {} ", format_date(*completed, false), format_date(*created, false)),
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
