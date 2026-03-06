use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::interaction::print_markdown;
use crate::projects::Project;
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
#[group(required = true, multiple = false)]
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
            let projects = Project::all(&mut file)?;
            for project in projects {
                println!("{}", project)
            }
            return Ok(())
        }

        let project = if let Some(project) = self.args.project.as_ref() {
            Project::new(project)
        } else if let Some(task_str) = self.args.task.as_ref() {
            let task = TodoTxtTask::parse(task_str)?;
            task.project()
                .ok_or(ToroError::ProjectNotFound())?
        } else {
            panic!()
        };

        let notes = project.notes()
            .ok().flatten()
            .unwrap_or_default();
        let tasks = file.iter()
            .filter(|t| t.project() == Some(project.clone()))
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        print_markdown("# Tasks");
        println!("{}\n", tasks);
        print_markdown(&notes);


        Ok(())
    }


    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
