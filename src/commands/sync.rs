use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args, Debug)]
pub struct SyncCommand {}

impl Command for SyncCommand {
    fn exec(self, _: Config) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        file.sync()
    }
}
