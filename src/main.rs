extern crate clap;
extern crate colored;
extern crate git2;
extern crate walkdir;

use std::env::current_dir;
use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;
use git2::Repository;
use walkdir::{DirEntry, WalkDir};

mod git;
mod input_args;

fn main() {
    let app = App::new("Git Governance")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("1.0")
        .subcommand(SubCommand::with_name("status")
            .arg(Arg::with_name("PATH")
                .short("f")
                .takes_value(true)
                .help("path of the directory from which the git governance will start analysing the git repos")
            )
        );

    let args = input_args::InputArgs::parse_inputs(app.get_matches());

    match args.input_command {
        input_args::InputCommand::Status => status(args.get_matches()),
        input_args::InputCommand::Create => create(),
        input_args::InputCommand::Error => {}
    }
}

fn create() {
//    Todo: to be implemented
}

fn status(matches: &ArgMatches) {
    match matches.value_of("PATH") {
        Some(path) => process_directories(path),
        None => {
            match current_dir() {
                Ok(dir) => {
                    match dir.to_str() {
                        Some(dir) => process_directories(dir),
                        None => panic!("Error in coverting current directory to string")
                    }
                }
                Err(err) => panic!("Error: {}", err),
            }
        }
    };
}

fn process_directories(path: &str) {
    let directory = WalkDir::new(path);

    for entry in directory
        .follow_links(true)
        .contents_first(true)
        .same_file_system(true)
        {
            match entry {
                Ok(directory) => process_directory(directory),
                Err(err) => panic!(err),
            };
        }
}

fn process_directory(dir: DirEntry) {
    if dir.file_name().eq(".git") {
        match dir.path().parent() {
            Some(dir) => process_git_directory(dir),
            None => println!("Error when getting parent directory of .git dir {:#?}", dir.path())
        };
    };
}

fn process_git_directory(path: &Path) {
    let repo = Repository::open(path);
    match repo {
        Ok(repo) => {
            match git::status(repo) {
                Ok(status) => print_repo_status(path, status),
                Err(err) => println!("Error: {} in processing git repository {:#?}", err, path)
            };
        }
        Err(err) => println!("Error: {} in opening git repository {:#?}", err, path)
    }
}

fn print_repo_status(path: &Path, statuses_in_dir: Vec<&str>) -> () {
    let mut statuses_in_dir = statuses_in_dir;
    if statuses_in_dir.is_empty() {
        println!("{:#?}: {}", path, "no changes".green());
    } else {
        statuses_in_dir.sort();
        statuses_in_dir.dedup();
        println!("{:#?}: {}", path, statuses_in_dir.join(", ").red());
    };
}