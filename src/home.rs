use std::fs;
use std::path::PathBuf;

use crate::error::{ToroError, ToroResult};
use crate::todotxt::TodoTxtFile;
use crate::Config;

const DATA_FILE_NAME: &str = "todo.txt";
const CONFIG_FILE_NAME: &str = "config.toml";

fn xdg_dirs() -> xdg::BaseDirectories {
    xdg::BaseDirectories::with_prefix("toro")
}

pub fn load_config() -> ToroResult<Option<Config>> {
    let path = match xdg_dirs().find_config_file(CONFIG_FILE_NAME) {
        Some(path) => path,
        None => return Ok(None),
    };

    let content = fs::read_to_string(&path)
        .map_err(|e| ToroError::NamedIOError(path.clone(), e))?;
    let config = toml::from_str(&content)?;
    Ok(config)
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
