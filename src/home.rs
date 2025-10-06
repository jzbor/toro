use std::path::PathBuf;

use crate::error::{ToroError, ToroResult};
use crate::todotxt::TodoTxtFile;

const DATA_FILE_NAME: &str = "todo.txt";

fn xdg_dirs() -> xdg::BaseDirectories {
    xdg::BaseDirectories::with_prefix("toro")
}

fn load_data_file() -> ToroResult<TodoTxtFile> {
    let path = xdg_dirs()
        .find_data_file(DATA_FILE_NAME)
        .ok_or(ToroError::DataFileNotFound())?;
    TodoTxtFile::load(path)
}

fn place_data_file() -> ToroResult<PathBuf> {
    xdg_dirs()
        .place_data_file(DATA_FILE_NAME)
        .map_err(ToroError::IOError)
}

pub fn load_or_create_data_file() -> ToroResult<TodoTxtFile> {
    load_data_file()
        .or_else(|_| place_data_file().map(TodoTxtFile::new))
}
