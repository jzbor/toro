use crate::commands::Command;
use crate::error::ToroResult;
use crate::filter::{ColumnSelector, Filter};
use crate::{home, Config};
use crate::interaction::announce;

#[derive(clap::Args, Debug)]
pub struct ViewCommand {
    /// Show numbers for all entries
    #[clap(short, long)]
    numbered: bool,

    /// Sort tasks from top to bottom
    #[clap(short, long)]
    top_to_bottom: bool,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    columns: ColumnSelector,
}

impl Command for ViewCommand {
    fn exec(self, config: Config) -> ToroResult<()> {
        let columns = config.columns.update_with_cmdline(self.columns);

        let file = home::load_or_create_data_file()?;
        announce(&format!("Tasks ({})", file.stats_fancy()));
        file.list(self.numbered, !self.top_to_bottom, columns, Some(&self.filter));
        Ok(())
    }
}
