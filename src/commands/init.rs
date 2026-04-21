
use std::fs;
use std::path::PathBuf;

use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::{Config, exec, home};

#[derive(clap::Args, Debug)]
pub struct InitCommand {
    /// Initialize from git repository
    #[clap(short, long)]
    git: Option<String>,

    /// Initialize by symlinking a data directory
    #[clap(short, long)]
    symlink: Option<PathBuf>,

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
        } else if let Some(symlink) = &self.symlink {
            let data_dir = home::propose_data_dir()?;

            // Workaround for `propose_data_dir()` returning directory path ending in `/`
            let data_dir = data_dir.parent()
                .ok_or(ToroError::DataDirInvalidCreation())?
                .join(data_dir.file_name().unwrap());

            data_dir.parent()
                .ok_or(ToroError::DataDirInvalidCreation())
                .and_then(|d| fs::create_dir_all(d).map_err(|e| e.into()))?;
            fs::create_dir_all(symlink)?;
            std::os::unix::fs::symlink(&symlink, &data_dir)?;
            println!("Created symlink ({} -> {})", data_dir.to_string_lossy(), symlink.to_string_lossy());
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
