use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args)]
pub struct RewriteCommand {
    #[clap(flatten)]
    config: Config,
}

impl Command for RewriteCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        file.store()?;

        if self.config.git.auto_commit {
            file.commit("Rewrite todo.txt file")?;
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

