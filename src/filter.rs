use chrono::Utc;
use clap::ArgAction;

use crate::todotxt::tasks::TodoTxtTask;


#[derive(clap::Args, Debug, Clone)]
pub struct Filter {
    // only ever read/use the non-negated versions

    /// Show completed entries
    #[clap(short, long, action = ArgAction::SetTrue, default_value_t = false)]
    pub today: bool,

    /// Hide completed entries
    #[clap(long, overrides_with = "today")]
    any_day: bool,

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


impl Filter {
    pub fn approves(&self, task: &TodoTxtTask) -> bool {
        if !self.include_completed && task.completed()  {
            return false;
        }

        if !self.include_pending && !task.completed()  {
            return false;
        }

        if self.today {
            let due = task.when_due().unwrap_or_default();
            let scheduled = task.when_scheduled().unwrap_or_default();
            let today = Utc::now().naive_local().into();
            match (due, scheduled) {
                (None, None) => return false,
                (None, Some(date)) => if date > today { return false },
                (Some(date), None) => if date > today { return false },
                (Some(date0), Some(date1)) => if date0 > today  && date1 > today { return false },
            }
        }

        let description = task.description();

        for pattern in &self.patterns {
            if !description.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }

        true
    }
}
