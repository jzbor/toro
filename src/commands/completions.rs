use std::fs;
use std::path;

use clap::CommandFactory;
use clap_complete::Shell;

use crate::error::ToroError;
use crate::error::ToroResult;
use crate::Config;


#[derive(clap::Args)]
pub struct CompletionsCommand {
    directory: path::PathBuf,
}

impl super::Command for CompletionsCommand {
    fn exec(self, _: Config) -> ToroResult<()> {
        let mut command = crate::Args::command();
        let shells = &[
            (Shell::Bash, "bash"),
            (Shell::Zsh, "zsh"),
            (Shell::Fish, "fish"),
            (Shell::PowerShell, "ps1"),
            (Shell::Elvish, "elv"),
        ];

        for (shell, ending) in shells {
            let mut file = fs::File::create(self.directory.join(format!("toro.{}", ending)))
                .map_err(|e| ToroError::CompletionsError(e.to_string()))?;
            clap_complete::aot::generate(*shell, &mut command, "toro", &mut file);
        }

        Ok(())
    }
}
