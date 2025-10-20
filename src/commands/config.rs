use crate::commands::Command;
use crate::error::ToroResult;
use crate::Config;

#[derive(clap::Args, Debug)]
pub struct ConfigCommand {
    #[clap(flatten)]
    config: Config,
}

impl Command for ConfigCommand {
    fn exec(&self) -> ToroResult<()> {
        println!("{}", toml::to_string_pretty(&self.config)?);
        Ok(())
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
