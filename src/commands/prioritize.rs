use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::interaction::{announce, select_tasks_mut};
use crate::{exec, home, Config};

#[derive(clap::Args, Debug)]
pub struct PrioritizeCommand {
    /// Priority for the task (uppercase character between 'A' and 'Z', '-' to remove any priority)
    prio: char,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}

impl Command for PrioritizeCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        let mut rl = rustyline::DefaultEditor::new()?;

        if let Some(cmd) = &self.config.view.cal_command {
            exec::exec("sh", ["-c", cmd])?
        }

        announce("Select tasks to prioritize");
        let res = select_tasks_mut(&mut rl, &mut file, &self.config, Some(&self.filter));
        let (_, mut selected) = match res {
            Ok(res) => res,
            Err(ToroError::EofError()) => return Ok(()),
            Err(e) => return Err(e),
        };
        let nselected = selected.len();

        if selected.is_empty() {
            return Ok(());
        }

        let (priority, none) = if self.prio.is_ascii_uppercase() {
            (self.prio, false)
        } else if self.prio == '-' {
            (self.prio, true)
        } else {
            return Err(ToroError::InvalidPriorityError('A'))
        };

        let new_val = if none { None } else { Some(priority) };
        selected.iter_mut()
            .for_each(|t| t.set_priority(new_val));

        println!("\nReprioritized {} task(s).\n", nselected);

        file.store()?;

        if self.config.git.auto_commit {
            file.commit(&format!("\nReprioritized {} task(s).\n", nselected))?;
        }
        if self.config.git.auto_sync {
            file.sync()?;
        };

        Ok(())
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
