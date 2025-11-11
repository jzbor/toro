use crate::commands::Command;
use crate::error::ToroResult;
use crate::filter::Filter;
use crate::{exec, home, Config};

#[derive(clap::Args, Debug)]
pub struct ViewCommand {
    /// Show numbers for all entries
    #[clap(short, long)]
    numbered: bool,

    /// Sort tasks from top to bottom
    #[clap(long)]
    top_to_bottom: bool,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}

impl Command for ViewCommand {
    fn exec(&self) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;

        if let Some(cmd) = &self.config.view.cal_command {
            exec::exec("sh", ["-c", cmd])?
        }

        file.list(self.numbered, !self.top_to_bottom, self.config.columns, &self.config.view, Some(&self.filter));
        Ok(())
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
