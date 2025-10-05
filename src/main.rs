use std::path::PathBuf;

use clap::Parser;

use crate::commands::Command;

mod home;
mod commands;
mod error;
mod todotxt;

/// CLI Client to manage a todo.txt list
#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// List all pending tasks
    List(commands::list::ListCommand)
}

fn main() {
    // let mut test = error::resolve(todotxt::TodoTxtFile::load(PathBuf::from("./test.txt")));
    // test.sort_by_key(|t| t.priority().unwrap_or('['));
    // test.sort_by_key(|t| t.completed());
    // println!();
    // for task in test.iter() {
    //     println!("{}", task.to_string_colored());
    // }

    // return;

    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        List(cmd) => cmd.exec(),
    };

    error::resolve(result)
}
