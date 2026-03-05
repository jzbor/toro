use colored::Colorize;

use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::{interaction::*, Config};
use crate::home;

#[derive(clap::Args)]
pub struct DoneCommand {
    #[clap(short, long)]
    multiple: bool,

    #[clap(short, long)]
    undo: bool,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}

impl Command for DoneCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        let mut filter = self.filter.clone();

        if !self.undo {
            filter.include_completed = false;
            filter.include_pending = true;
        } else {
            filter.include_completed = true;
            filter.include_pending = false;
        }
        let prompt = if !self.undo {
            "Select tasks to mark as completed"
        } else {
            "Select tasks to mark as pending"
        };

        let res = select_tasks_mut(&mut file, &self.config, Some(&self.filter), prompt);
        let (_, selected) = match res {
            Ok(res) => res,
            Err(ToroError::EofError()) => return Ok(()),
            Err(e) => return Err(e),
        };
        let nselected = selected.len();

        println!();

        for task in selected {
            if !self.undo {
                println!("Marking \"{}\" as {}.",
                    task.description_fancy(&self.config.view).color(SELECTION_COLOR),
                    "completed".color(COMPLETED_COLOR));
                task.complete();
            } else {
                println!("Marking \"{}\" as {}.",
                    task.description_fancy(&self.config.view).color(SELECTION_COLOR),
                    "pending".color(PENDING_COLOR));
                task.uncomplete();
            }
        }

        file.store()?;

        if self.config.git.auto_commit {
            let state = if self.undo { "pending" } else { "completed" };
            file.commit(&format!("Marked {} tasks as \"{}\"", nselected, state))?;
        }
        if self.config.git.auto_sync {
            file.sync()?;
        }

        Ok(())
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

