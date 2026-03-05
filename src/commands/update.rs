use colored::Colorize;
use chrono::{Datelike, NaiveDate};

use crate::commands::Command;
use crate::error::{complain, ToroError, ToroResult};
use crate::filter::Filter;
use crate::{exec, interaction::*};
use crate::date::parse_date;
use crate::{home, Config};

#[derive(clap::Args, Debug)]
pub struct UpdateCommand {
    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}

impl Command for UpdateCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;

        loop {
            if let Some(cmd) = &self.config.view.cal_command {
                exec::exec("sh", ["-c", cmd])?
            }

            let res = select_tasks_mut(&mut file, &self.config, Some(&self.filter), "Select task(s) to update: ");
            let (auto_selected, mut selected) = match res {
                Ok(res) => res,
                Err(ToroError::EofError()) => return Ok(()),
                Err(e) => return Err(e),
            };
            let nselected = selected.len();

            if selected.is_empty() {
                continue;
            }

            println!();
            print_header("Selected tasks:", 1);
            let borrowed: Vec<_> = selected.iter().map(|s| &**s).collect();
            list_tasks(&borrowed, None, self.config.columns, &self.config.view);

            println!();

            let field = match select_field() {
                Ok(field) => field,
                Err(ToroError::EofError()) => return Ok(()),
                Err(e) => return Err(e),
            };

            use FieldSelection::*;
            let mut previous_values = match field {
                Completed => selected.iter()
                    .map(|t| t.completed().to_string().color(field.color()).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                Priority => selected.iter()
                    .map(|t| t.priority()
                        .map(|p| format!("{}", p))
                        .unwrap_or("none".to_string())
                        .color(field.color())
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                CompletionDate => selected.iter()
                    .map(|t| t.when_completed()
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .color(field.color())
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                CreationDate => selected.iter()
                    .map(|t| t.when_created()
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .color(field.color())
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                Description => selected.iter()
                    .map(|t| t.description().color(field.color()).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                Due => selected.iter()
                    .map(|t| t.when_due()
                        .unwrap_or(None)
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .color(field.color())
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                Scheduled => selected.iter()
                    .map(|t| t.when_scheduled()
                        .unwrap_or(None)
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .color(field.color())
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            };
            if previous_values.len() > 68 {
                previous_values = previous_values[..65].to_string();
                previous_values.push_str("...");
            }

            let first_previous = match field {
                Completed => selected.first()
                    .map(|t| t.completed().to_string())
                    .unwrap_or(String::new()),
                Priority => selected.first()
                    .map(|t| t.priority()
                        .map(|p| format!("{}", p))
                        .unwrap_or("none".to_string())
                        .to_string())
                    .unwrap_or(String::new()),
                CompletionDate => selected.first()
                    .map(|t| t.when_completed()
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .to_string())
                    .unwrap_or(String::new()),
                CreationDate => selected.first()
                    .map(|t| t.when_created()
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .to_string())
                    .unwrap_or(String::new()),
                Description => selected.first()
                    .map(|t| t.description())
                    .unwrap_or(String::new()),
                Due => selected.first()
                    .map(|t| t.when_due()
                        .unwrap_or(None)
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .to_string())
                    .unwrap_or(String::new()),
                Scheduled => selected.first()
                    .map(|t| t.when_scheduled()
                        .unwrap_or(None)
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .to_string())
                    .unwrap_or(String::new()),
            };


            let mut rl = rustyline::DefaultEditor::new().unwrap();

            loop {
                println!();
                println!("Old value: {}", previous_values);
                let answer = match rl.readline_with_initial("New value: ", (&first_previous, "")) {   // TODO
                // let answer = match rl.readline("New value: ") {
                    Ok(answer) => answer,
                    Err(rustyline::error::ReadlineError::Eof) => return Ok(()),
                    Err(e) => return Err(e.into()),
                };

                match field {
                    Completed => {
                        let completed: bool = match answer.parse() {
                            Ok(completed) => completed,
                            Err(_) => { complain(ToroError::InvalidValue(answer, field)); continue },
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_completed(completed))
                    },
                    Priority => {
                        let priority: Option<char> = if answer == "none" {
                            None
                        } else {
                            let prio: char = match answer.parse() {
                                Ok(priority) => priority,
                                Err(_) => { complain(ToroError::InvalidValue(answer.clone(), field)); continue },
                            };
                            if !prio.is_ascii_uppercase() {
                                complain(ToroError::InvalidValue(answer.clone(), field));
                                continue
                            } else {
                                Some(prio)
                            }
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_priority(priority))
                    },
                    CompletionDate => {
                        let date: Option<NaiveDate> = if answer == "none" {
                            None
                        } else {
                            match parse_date(&answer) {
                                Ok(date) => Some(date),
                                Err(_) => { complain(ToroError::InvalidValue(answer, field)); continue },
                            }
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_completion_date(date))
                    },
                    CreationDate => {
                        let date: Option<NaiveDate> = if answer == "none" {
                            None
                        } else {
                            match parse_date(&answer) {
                                Ok(date) => Some(date),
                                Err(_) => { complain(ToroError::InvalidValue(answer, field)); continue },
                            }
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_creation_date(date))
                    },
                    Description => {
                        let res: ToroResult<()> = selected.iter_mut().try_for_each(|t| t.set_description(&answer));
                        if res.is_err() {
                            complain(ToroError::InvalidValue(answer, field));
                            continue;
                        }
                    },
                    Due => {
                        let date: Option<NaiveDate> = if answer == "none" {
                            None
                        } else {
                            match parse_date(&answer) {
                                Ok(date) => Some(date),
                                Err(_) => { complain(ToroError::InvalidValue(answer, field)); continue },
                            }
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_due(date))
                    },
                    Scheduled => {
                        let date: Option<NaiveDate> = if answer == "none" {
                            None
                        } else {
                            match parse_date(&answer) {
                                Ok(date) => Some(date),
                                Err(_) => { complain(ToroError::InvalidValue(answer, field)); continue },
                            }
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_scheduled(date))
                    },
                };

                break;
            }

            println!("\nUpdated {} in {} task(s).\n", field.to_string_fancy(), nselected);

            file.store()?;

            if self.config.git.auto_commit {
                file.commit(&format!("Updated {} in {} task(s)", field, nselected))?;
            }
            if self.config.git.auto_sync {
                file.sync()?;
            }

            if auto_selected {
                return Ok(())
            }
        }
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
