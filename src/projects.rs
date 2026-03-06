use std::{env, fs};
use std::fmt::Display;
use std::path::PathBuf;

use crate::error::{ToroError, ToroResult};
use crate::exec::exec;
use crate::{NOTE_DIR, home};
use crate::todotxt::file::TodoTxtFile;


#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Project(String);


impl Project {
    pub fn new(name: &str) -> Self {
        let name = name.strip_prefix("+").unwrap_or(name);
        Project(name.to_owned())
    }

    pub fn all(file: &mut TodoTxtFile) -> ToroResult<Vec<Self>> {
        let mut projects = Vec::new();
        let task_projects = file.iter().flat_map(|t| t.project().map(|p| p.to_owned()));
        let note_projects = notes()?.into_iter()
            .map(|f| f.file_name().unwrap_or_default().to_string_lossy().to_string())
            .map(|f| Self::new(f.strip_suffix(".md").unwrap_or(&f)));

        projects.extend(task_projects);
        projects.extend(note_projects);

        projects.sort();
        projects.dedup();

        Ok(projects)
    }

    pub fn edit_notes(&self) -> ToroResult<PathBuf> {
        let xdg = home::xdg_dirs();
        let note_file = xdg.place_data_file(PathBuf::from(NOTE_DIR).join(format!("{}.md", self.0)))?;
        let editor = env::var("EDITOR")
            .map_err(|_| ToroError::MissingEnvVar("EDITOR"))?;

        exec(&editor, [note_file.to_string_lossy().to_string()])?;

        Ok(note_file)
    }

    pub fn notes(&self) -> ToroResult<Option<String>> {
        let xdg = home::xdg_dirs();
        match xdg.get_data_file(PathBuf::from(NOTE_DIR).join(format!("{}.md", self.0))) {
            Some(file) => fs::read_to_string(file).map(Some).map_err(|e| e.into()),
            None => Ok(None),
        }
    }
}

pub fn notes() -> ToroResult<Vec<PathBuf>> {
    let xdg = home::xdg_dirs();
    let note_dir = match xdg.get_data_file(PathBuf::from(NOTE_DIR)) {
        Some(dir) => dir,
        None => return Ok(Vec::new()),
    };
    let note_files = note_dir.read_dir()?
        .flatten()
        .filter(|f| f.file_name().to_string_lossy().ends_with(".md"))
        .map(|f| note_dir.join(f.file_name()))
        .collect();

    Ok(note_files)
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
