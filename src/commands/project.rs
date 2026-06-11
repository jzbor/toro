use std::cmp::Reverse;

use chrono::NaiveDate;

use crate::commands::Command;
use crate::config::{SortBy, ViewConfig};
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::interaction::{inc_markdown_headers, print_markdown};
use crate::projects::Project;
use crate::todotxt::file::TodoTxtFile;
use crate::todotxt::tasks::TodoTxtTask;
use crate::{home, Config};


#[derive(clap::Args, Debug)]
pub struct ProjectCommand {
    #[clap(flatten)]
    args: ProjectArgs,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}

#[derive(clap::Args, Debug)]
#[group(required = false, multiple = false)]
struct ProjectArgs {
    /// Name of the project
    project: Option<String>,

    /// Use project from task
    #[clap(long)]
    task: Option<String>,

    /// List projects
    #[clap(short, long)]
    list: bool,
}


impl Command for ProjectCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;

        if self.args.list {
            for project in Project::all(&mut file)? {
                println!("{}", project)
            }

            return Ok(())
        } else if self.args.project.is_some() || self.args.task.is_some() {
            let project = if let Some(project) = self.args.project.as_ref() {
                Project::new(project)
            } else if let Some(task_str) = self.args.task.as_ref() {
                let task = TodoTxtTask::parse(task_str)?;
                task.project()
                    .ok_or(ToroError::ProjectNotFound())?
            } else {
                panic!()
            };

            let notes = project.notes().ok().flatten();
            let tasks: Vec<_> = project.tasks(&file, &self.filter, &self.config.view.sort).collect();
            show_project(&project, notes, &tasks, &self.config);

        } else {
            let mut projects: Vec<(_, _)> = Project::all(&mut file)?
                .into_iter()
                .map(|p| {
                    let tasks = p.tasks(&file, &self.filter, &self.config.view.sort).collect::<Vec<_>>();
                    (p, tasks)
                })
                .collect();

            projects.sort_by_key(|(_, ts)| ts.len());
            projects.sort_by_key(|(_, ts)| Reverse(ts.iter()
                    .map(|t| t.priority().unwrap_or(char::MAX))
                    .min().unwrap_or(char::MAX)));
            projects.sort_by_key(|(_, ts)| Reverse(ts.iter()
                    .map(|t| t.when_due().ok().flatten().unwrap_or(NaiveDate::MAX))
                    .min().unwrap_or(NaiveDate::MAX)));
            projects.sort_by_key(|(_, ts)| Reverse(ts.iter()
                    .map(|t| t.when_scheduled().ok().flatten().unwrap_or(NaiveDate::MAX))
                    .min().unwrap_or(NaiveDate::MAX)));
            // projects.sort_by_key(|(_, ts)| Reverse(ts.iter().flat_map(|t| t.when_due().ok().flatten()).min()));

            for (project, tasks) in &projects {
                let notes = project.notes().ok().flatten();

                if notes.is_some() || !tasks.is_empty() {
                    show_project(project, notes, tasks, &self.config);
                    println!();
                }
            }
        }

        Ok(())
    }


    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

fn show_project(project: &Project, notes: Option<String>, tasks: &[&TodoTxtTask], config: &Config) {
    let tasks = tasks.into_iter()
        .filter(|t| t.project() == Some(project.clone()))
        .map(|t| t.to_string_fancy(config.columns, &config.view))
        .map(|s| format!("- {}", s.trim()))
        .collect::<Vec<_>>()
        .join("\n");

    print_markdown(&format!("# {}", project.name()));

    if let Some(notes) = notes {
        print_markdown(&inc_markdown_headers(&notes));
    }

    if tasks.len() > 0 {
        println!();
        print_markdown("## Tasks");
        println!("{}", tasks);
    }
}
