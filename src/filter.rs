use clap::ArgAction;

use crate::todotxt::TodoTxtTask;


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


impl Filter {
    pub fn approves(&self, task: &TodoTxtTask) -> bool {
        if !self.include_completed && task.completed()  {
            return false;
        }

        if !self.include_pending && !task.completed()  {
            return false;
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
