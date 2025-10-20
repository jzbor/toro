use colored::Colorize;
use chrono::{Datelike, NaiveDate};

use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::{exec, interaction::*};
use crate::todotxt::parse_date;
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
        let mut rl = rustyline::DefaultEditor::new()?;

        loop {
            if let Some(cmd) = &self.config.view.cal_command {
                exec::exec("sh", ["-c", cmd])?
            }

            announce("Select tasks to update");
            let nrs = match select_tasks(&mut rl, &file, self.config.columns, Some(&self.filter)) {
                Ok(nrs) => nrs,
                Err(ToroError::EofError()) => return Ok(()),
                Err(e) => return Err(e),
            };

            let filtered = file.filtered_tasks_mut(&self.filter);
            let mut selected: Vec<_> = filtered.into_iter()
                .enumerate()
                .filter_map(|(i, t)| if nrs.contains(&i) { Some(t) } else { None })
                .collect();

            if selected.is_empty() {
                continue;
            }

            println!();

            let field = match select_field(&mut rl) {
                Ok(field) => field,
                Err(ToroError::EofError()) => return Ok(()),
                Err(e) => return Err(e),
            };

            use FieldSelection::*;
            let mut previous_values = match field {
                Completed => selected.iter()
                    .map(|t| t.completed().to_string().color(COMPLETED_COLOR).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                Priority => selected.iter()
                    .map(|t| t.priority()
                        .map(|p| format!("{}", p))
                        .unwrap_or("none".to_string())
                        .color(PRIORITY_COLOR)
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                CompletionDate => selected.iter()
                    .map(|t| t.when_completed()
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .color(COMPLETION_DATE_COLOR)
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                CreationDate => selected.iter()
                    .map(|t| t.when_created()
                        .map(|p| format!("{:0>4}-{:0>2}-{:0>2}", p.year(), p.month(), p.day()))
                        .unwrap_or("none".to_string())
                        .color(CREATION_DATE_COLOR)
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                Description => selected.iter()
                    .map(|t| t.description().color(DESCRIPTION_COLOR).to_string())
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
                            Err(_) => { eprintln!("Invalid value \"{}\" for field {}", answer, field); continue },
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_completed(completed))
                    },
                    Priority => {
                        let priority: Option<char> = if answer == "none" {
                            None
                        } else {
                            match answer.parse() {
                                Ok(priority) => Some(priority),
                                Err(_) => { eprintln!("Invalid value \"{}\" for field {}", answer, field); continue },
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
                                Err(_) => { eprintln!("Invalid value \"{}\" for field {}", answer, field); continue },
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
                                Err(_) => { eprintln!("Invalid value \"{}\" for field {}", answer, field); continue },
                            }
                        };
                        selected.iter_mut()
                            .for_each(|t| t.set_creation_date(date))
                    },
                    Description => {
                        let res: ToroResult<()> = selected.iter_mut().try_for_each(|t| t.set_description(&answer));
                        if res.is_err() {
                            eprintln!("Invalid input \"{}\" for description", answer);
                            continue;
                        }
                    },
                };

                break;
            }

            println!("\nUpdated {} in {} task(s).\n", field.to_string_fancy(), nrs.len());

            file.store()?;

            if self.config.git.auto_commit {
                file.commit(&format!("Updated {} in {} task(s)", field, nrs.len()))?;
            }
            if self.config.git.auto_sync {
                file.sync()?;
            }
        }

    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
