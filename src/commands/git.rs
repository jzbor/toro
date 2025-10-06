use crate::commands::Command;
use crate::error::ToroResult;
use crate::{home, Config};

#[derive(clap::Args, Debug)]
pub struct GitCommand {
    #[clap(allow_hyphen_values = true)]
    args: Vec<String>,
}

impl Command for GitCommand {
    fn exec(self, _: Config) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        file.git(self.args)
    }
}
