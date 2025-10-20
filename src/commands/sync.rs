use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args, Debug)]
pub struct SyncCommand {
    #[clap(flatten)]
    config: Config,
}

impl Command for SyncCommand {
    fn exec(&self) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        file.sync()
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
