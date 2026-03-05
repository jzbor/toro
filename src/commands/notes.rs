use std::env;
use std::path::PathBuf;

use crate::commands::Command;
use crate::error::{ToroError, ToroResult};
use crate::filter::Filter;
use crate::interaction::select_tasks_mut;
use crate::todotxt::TodoTxtFile;
use crate::{exec::exec, home, Config};


const NOTE_DIR: &str = "notes";


#[derive(clap::Args, Debug)]
pub struct NotesCommand {
    /// Name of the project
    project: Option<String>,

    #[clap(flatten)]
    filter: Filter,

    #[clap(flatten)]
    config: Config,
}


impl Command for NotesCommand {
    fn exec(&self) -> ToroResult<()> {
        let mut file = home::load_or_create_data_file()?;

        if let Some(project) = &self.project {
            edit(&mut file, project)
        } else {
            loop {
                let res = select_tasks_mut(&mut file, &self.config, Some(&self.filter), "Select project to open notes: ");
                let (_, selected) = match res {
                    Ok(res) => res,
                    Err(ToroError::EofError()) => return Ok(()),
                    Err(e) => return Err(e),
                };

                if selected.is_empty() {
                    continue;
                }

                let mut projects: Vec<_> = selected.into_iter()
                    .flat_map(|t| t.project())
                    .collect();
                projects.sort();
                projects.dedup();

                let project = match projects.as_slice() {
                    &[p] => p.to_owned(),
                    _ => { eprintln!("Project ambiguous"); continue },
                };

                edit(&mut file, &project)?;
            }
        }
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

fn edit(file: &mut TodoTxtFile, project: &str) -> ToroResult<()> {
    let xdg = home::xdg_dirs();
    let project = project.strip_prefix("+").unwrap_or(project);
    let note_file = xdg.place_data_file(PathBuf::from(NOTE_DIR).join(format!("{}.md", project)))?;
    let editor = env::var("EDITOR")
        .map_err(|_| ToroError::MissingEnvVar("EDITOR"))?;

    exec(&editor, [note_file.to_string_lossy().to_string()])?;

    if note_file.exists() {
        file.git(["add", note_file.to_string_lossy().as_ref()])?;
        file.commit("Updated notes")?;
    }

    Ok(())
}
