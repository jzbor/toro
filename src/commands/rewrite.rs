use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args)]
pub struct RewriteCommand {
}

impl Command for RewriteCommand {
    fn exec(self, config: Config) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        file.store()
    }
}

