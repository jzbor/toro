use colored::Colorize;

use crate::commands::Command;
use crate::error::ToroResult;
use crate::filter::{ColumnSelector, Filter};
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
    columns: ColumnSelector,
}

impl Command for DoneCommand {
    fn exec(mut self, config: Config) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        let columns = config.columns.update_with_cmdline(self.columns);

        if !self.undo {
            self.filter.include_completed = false;
            self.filter.include_pending = true;
            announce("Select tasks to mark as completed");
        } else {
            self.filter.include_completed = true;
            self.filter.include_pending = false;
            announce("Select tasks to mark as pending");
        }

        let nrs = select_tasks(&file, columns, Some(&self.filter));
        let mut filtered = file.filtered_tasks_mut(&self.filter);

        let selected = filtered.iter_mut()
            .enumerate()
            .filter_map(|(i, t)| if nrs.contains(&i) { Some(t) } else { None });

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

        Ok(())
    }
}

