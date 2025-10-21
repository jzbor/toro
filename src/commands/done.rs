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
        let mut rl = rustyline::DefaultEditor::new()?;
        let mut filter = self.filter.clone();

        if !self.undo {
            filter.include_completed = false;
            filter.include_pending = true;
            announce("Select tasks to mark as completed");
        } else {
            filter.include_completed = true;
            filter.include_pending = false;
            announce("Select tasks to mark as pending");
        }

        let res = select_tasks(&mut rl, &file, self.config.columns, Some(&self.filter), self.config.view.auto_select);
        let (_, nrs) = match res {
            Ok(nrs) => nrs,
            Err(ToroError::EofError()) => return Ok(()),
            Err(e) => return Err(e),
        };
        let mut filtered = file.filtered_tasks_mut(&self.filter);

        let selected = filtered.iter_mut()
            .enumerate()
            .filter_map(|(i, t)| if nrs.contains(&i) { Some(t) } else { None });

        println!();

        for task in selected {
            if !self.undo {
                println!("Marking \"{}\" as {}.", task.description_fancy().color(SELECTION_COLOR), "completed".color(COMPLETED_COLOR));
                task.complete();
            } else {
                println!("Marking \"{}\" as {}.", task.description_fancy().color(SELECTION_COLOR), "pending".color(PENDING_COLOR));
                task.uncomplete();
            }
        }

        file.store()?;

        if self.config.git.auto_commit {
            let state = if self.undo { "pending" } else { "completed" };
            file.commit(&format!("Marked {} tasks as \"{}\"", nrs.len(), state))?;
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

