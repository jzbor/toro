use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args)]
pub struct RewriteCommand {
}

impl Command for RewriteCommand {
    fn exec(self, config: Config) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        file.store()?;

        if config.git.auto_commit {
            file.commit("Rewrite todo.txt file")?;
        }
        if config.git.auto_sync {
            file.sync()?;
        }


        Ok(())
    }
}

