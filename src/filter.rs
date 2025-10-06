use clap::ArgAction;

use crate::todotxt::TodoTxtTask;


#[derive(clap::Args, serde::Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ColumnSelector {
    // only ever read/use the non-negated versions

    /// Show completion mark column
    #[clap(long = "col-completed")]
    #[serde(default = "crate::home::serde_bool_true")]
    pub completed: bool,

    /// Hide completion mark column
    #[clap(long = "no-col-completed", overrides_with = "completed")]
    #[serde(skip)]
    no_completed: bool,

    /// Show priority column
    #[clap(long = "col-priority")]
    #[serde(default = "crate::home::serde_bool_true")]
    pub priority: bool,

    /// Hide priority column
    #[clap(long = "no-col-priority", overrides_with = "priority")]
    #[serde(skip)]
    no_priority: bool,

    /// Show completion date column
    #[clap(long = "col-completion-date")]
    #[serde(default = "crate::home::serde_bool_true")]
    pub completion_date: bool,

    /// Hide completion date column
    #[clap(long = "no-col-completion-date", overrides_with = "completion_date")]
    #[serde(skip)]
    no_completion_date: bool,

    /// Show creation date column
    #[clap(long = "col-creation-date")]
    #[serde(default = "crate::home::serde_bool_true")]
    pub creation_date: bool,

    /// Hide creation date column
    #[clap(long = "no-col-creation-date", overrides_with = "creation_date")]
    #[serde(skip)]
    no_creation_date: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct Filter {
    // only ever read/use the non-negated versions

    /// Show completed entries
    #[clap(long, action = ArgAction::SetTrue, default_value_t = false)]
    pub include_completed: bool,

    /// Hide completed entries
    #[clap(long, overrides_with = "include_completed")]
    exclude_completed: bool,

    /// Show pending entries
    #[clap(long, action = ArgAction::SetTrue, default_value_t = true)]
    pub include_pending: bool,

    /// Hide pending entries
    #[clap(long, overrides_with = "include_pending")]
    exclude_pending: bool,

    /// Additional search patterns
    patterns: Vec<String>
}


impl ColumnSelector {
    pub fn update_with_cmdline(mut self, other: Self) -> Self {
        if other.completed || other.no_completed {
            self.completed = other.completed
        }

        if other.priority || other.no_priority {
            self.priority = other.priority
        }

        if other.completion_date || other.no_completion_date {
            self.completion_date = other.completion_date
        }

        if other.creation_date || other.no_creation_date {
            self.creation_date = other.creation_date
        }

        self
    }
}

impl Filter {
    pub fn update_with_cmdline(mut self, other: Self) -> Self {
        if other.include_completed || other.exclude_completed {
            self.include_completed = other.exclude_completed
        }

        if other.include_pending || other.exclude_pending {
            self.include_pending = other.exclude_pending
        }

        self
    }

    pub fn approves(&self, task: &TodoTxtTask) -> bool {
        if !self.include_completed && task.completed()  {
            return false;
        }

        if !self.include_pending && !task.completed()  {
            return false;
        }

        let description = task.description();

        for pattern in &self.patterns {
            if !description.contains(pattern) {
                return false;
            }
        }

        true
    }
}

impl Default for ColumnSelector {
    fn default() -> Self {
        ColumnSelector {
            completed: true,
            no_completed: false,
            priority: true,
            no_priority: false,
            completion_date: true,
            no_completion_date: false,
            creation_date: true,
            no_creation_date: false
        }
    }
}
