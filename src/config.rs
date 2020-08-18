use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{Error as GitError, Repository};

use crate::input_args::InputArgs;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("config")
        .arg(
            Arg::with_name("root_path")
                .short("r")
                .default_value(".")
                .takes_value(true)
                .help("path of the repo where branches are to be compared. Defaults to '.'"),
        )
}

pub fn config(args: InputArgs) {
    let root_path = args.get_root_path("root_path");
    let repo = Repository::open(root_path).expect("Failed to open git repo");
    print!("{}", "[TBD] Config Creation ...".blue());
}