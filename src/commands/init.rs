
use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::{Config, exec, home};

#[derive(clap::Args, Debug)]
pub struct InitCommand {
    /// Initialize from git repository
    #[clap(short, long)]
    git: Option<String>,

    #[clap(flatten)]
    config: Config,
}

impl Command for InitCommand {
    fn exec(&self) -> ToroResult<()> {
        if let Ok(file) = home::load_data_file() {
            Err(ToroError::DataFileExists(file.location().to_string_lossy().to_string()))
        } else if let Some(repo) = &self.git {
            let data_dir = home::propose_data_dir()?
                .to_string_lossy()
                .to_string();
            exec::exec("git", ["clone", "--recurse-submodules", repo.as_str(), data_dir.as_str()])?;
            let file = home::load_data_file()?;
            println!("Initialized data file ({}) from \"{}\"", file.location().to_string_lossy(), repo);
            Ok(())
        } else {
            let file = home::place_data_file()?;
            println!("Created data file ({})", file.to_string_lossy());
            Ok(())
        }
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
