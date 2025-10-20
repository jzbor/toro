use colored::Colorize;

use crate::commands::Command;
use crate::error::ToroResult;
use crate::todotxt::TodoTxtTask;
use crate::{home, Config};
use crate::interaction::*;

#[derive(clap::Args, Debug)]
pub struct NewCommand {
    /// New task in todo.txt format
    text: String,

    #[clap(flatten)]
    config: Config,
}

impl Command for NewCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        let task = TodoTxtTask::parse(&self.text)?;
        let description = task.description().color(SELECTION_COLOR);
        let description_fancy = task.description_fancy().color(SELECTION_COLOR);

        println!("Creating new task \"{}\".", description_fancy);

        file.push(task);
        file.store()?;

        if self.config.git.auto_commit {
            file.commit(&format!("Created new task \"{}\"", description))?;
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
