use chrono::NaiveDate;

use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::{exec, interaction::*};
use crate::date::parse_date;
use crate::{home, Config};

#[derive(clap::Args, Debug)]
pub struct SetCommand {
    #[clap(skip)]
    field: Option<FieldSelection>,

    /// Value to set
    value: String,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}


impl SetCommand {
    pub fn with_field(mut self, field: FieldSelection) -> Self {
        self.field = Some(field);
        self
    }
}

impl Command for SetCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;
        let field = match self.field {
            Some(f) => f,
            None => panic!("`field` not set"),
        };

        if let Some(cmd) = &self.config.view.cal_command {
            exec::exec("sh", ["-c", cmd])?
        }

        let res = select_tasks_mut(&mut file, &self.config, Some(&self.filter), "Select task(s) to update: ");
        let (_, mut selected) = match res {
            Ok(res) => res,
            Err(ToroError::EofError()) => return Ok(()),
            Err(e) => return Err(e),
        };
        let nselected = selected.len();

        if selected.is_empty() {
            return Ok(());
        }

        println!();

        use FieldSelection::*;
        match field {
            Completed => {
                let completed: bool = match self.value.parse() {
                    Ok(completed) => completed,
                    Err(_) => return Err(ToroError::InvalidValue(self.value.clone(), field)),
                };
                selected.iter_mut()
                    .for_each(|t| t.set_completed(completed))
            },
            Priority => {
                let priority: Option<char> = if self.value == "none" {
                    None
                } else {
                    let prio: char = match self.value.parse() {
                        Ok(priority) => priority,
                        Err(_) => return Err(ToroError::InvalidValue(self.value.clone(), field)),
                    };
                    if !prio.is_ascii_uppercase() {
                        return Err(ToroError::InvalidValue(self.value.clone(), field));
                    } else {
                        Some(prio)
                    }
                };

                selected.iter_mut()
                    .for_each(|t| t.set_priority(priority))
            },
            CompletionDate => {
                let date: Option<NaiveDate> = if self.value == "none" {
                    None
                } else {
                    match parse_date(&self.value) {
                        Ok(date) => Some(date),
                        Err(_) => return Err(ToroError::InvalidValue(self.value.clone(), field)),
                    }
                };
                selected.iter_mut()
                    .for_each(|t| t.set_completion_date(date))
            },
            CreationDate => {
                let date: Option<NaiveDate> = if self.value == "none" {
                    None
                } else {
                    match parse_date(&self.value) {
                        Ok(date) => Some(date),
                        Err(_) => return Err(ToroError::InvalidValue(self.value.clone(), field)),
                    }
                };
                selected.iter_mut()
                    .for_each(|t| t.set_creation_date(date))
            },
            Description => {
                let res: ToroResult<()> = selected.iter_mut().try_for_each(|t| t.set_description(&self.value));
                if res.is_err() {
                    return Err(ToroError::InvalidValue(self.value.clone(), field))
                }
            },
            Due => {
                let date: Option<NaiveDate> = if self.value == "none" {
                    None
                } else {
                    match parse_date(&self.value) {
                        Ok(date) => Some(date),
                        Err(_) => return Err(ToroError::InvalidValue(self.value.clone(), field)),
                    }
                };
                selected.iter_mut()
                    .for_each(|t| t.set_due(date))
            },
            Scheduled => {
                let date: Option<NaiveDate> = if self.value == "none" {
                    None
                } else {
                    match parse_date(&self.value) {
                        Ok(date) => Some(date),
                        Err(_) => return Err(ToroError::InvalidValue(self.value.clone(), field)),
                    }
                };
                selected.iter_mut()
                    .for_each(|t| t.set_scheduled(date))
            },
        };

        println!("\nUpdated {} in {} task(s).\n", field.to_string_fancy(), nselected);

        file.store()?;

        if self.config.git.auto_commit {
            file.commit(&format!("Updated {} in {} task(s)", field, nselected))?;
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
