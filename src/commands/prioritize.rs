use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::interaction::{announce, select_tasks};
use crate::{exec, home, Config};

#[derive(clap::Args, Debug)]
pub struct PrioritizeCommand {
    #[clap(long)]
    high: bool,

    #[clap(long)]
    low: bool,

    #[clap(short, long)]
    none: bool,

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
        let res = select_tasks(&mut rl, &file, self.config.columns, Some(&self.filter), self.config.view.auto_select);
        let (_, nrs) = match res {
            Ok(nrs) => nrs,
            Err(ToroError::EofError()) => return Ok(()),
            Err(e) => return Err(e),
        };

        let filtered = file.filtered_tasks_mut(&self.filter);
        let mut selected: Vec<_> = filtered.into_iter()
            .enumerate()
            .filter_map(|(i, t)| if nrs.contains(&i) { Some(t) } else { None })
            .collect();

        if selected.is_empty() {
            return Ok(());
        }

        let priority = if self.high {
            Some('A')
        } else if self.low {
            Some('Z')
        } else if self.none {
            None
        } else {
            Some('B')
        };

        selected.iter_mut()
            .for_each(|t| t.set_priority(priority));

        println!("\nReprioritized {} task(s).\n",  nrs.len());

        file.store()?;

        if self.config.git.auto_commit {
            file.commit(&format!("\nReprioritized {} task(s).\n",  nrs.len()))?;
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
