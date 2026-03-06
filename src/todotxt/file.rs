use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use chrono::{NaiveDate, TimeDelta, Utc};


use crate::config::{ColumnSelector, SortBy, ViewConfig};
use crate::todotxt::tasks::TodoTxtTask;
use crate::interaction;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::exec::*;


#[derive(Debug)]
pub struct TodoTxtFile {
    location: PathBuf,
    tasks: Vec<TodoTxtTask>,
}

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
        let mut tasks: Vec<_> = self.filtered_sorted(filter_opt, &view.sort).collect();

        let ntasks = tasks.len();
        let numbering = match (numbered, reverse) {
            (true, false) => Some((0..tasks.len()).collect::<Vec<_>>()),
            (true, true) => Some((0..tasks.len()).rev().collect::<Vec<_>>()),
            (false, _) => None,
        };
        if reverse {
            tasks.reverse();
        }

        interaction::list_tasks(&tasks, numbering.as_deref(), columns, view);

        ntasks
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

