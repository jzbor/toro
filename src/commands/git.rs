use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args, Debug)]
pub struct GitCommand {
    #[clap(allow_hyphen_values = true)]
    args: Vec<String>,

    #[clap(flatten)]
    config: Config,
}

impl Command for GitCommand {
    fn exec(&self) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        file.git(self.args.iter())
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
