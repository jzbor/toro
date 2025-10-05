use crate::commands::Command;
use crate::error::ToroResult;
use crate::home;

#[derive(clap::Args)]
pub struct ListCommand {
}

impl Command for ListCommand {
    fn exec(self) -> ToroResult<()> {
        let file = home::load_or_create_data_file()?;
        for task in file.iter() {
            println!("{}", task.to_string_colored());
        }
        Ok(())
    }
}
