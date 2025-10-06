use std::env;

use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::exec::exec;
use crate::{home, Config};

#[derive(clap::Args)]
pub struct EditCommand {
}

impl Command for EditCommand {
    fn exec(self, config: Config) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        let path = file.location();
        let editor = env::var("EDITOR")
            .map_err(|_| ToroError::MissingEnvVar("EDITOR"))?;

        exec(&editor, [path])?;

        Ok(())
    }
}

