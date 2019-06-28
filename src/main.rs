extern crate clap;
extern crate colored;
extern crate git2;
extern crate walkdir;

use std::env::current_dir;
use std::path::Path;

use clap::{App, AppSettings, Arg, SubCommand};
use git2::Repository;
use walkdir::{DirEntry, WalkDir};

mod git;

fn main() {
    let matches = App::new("Git Governance")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("1.0")
        .subcommand(SubCommand::with_name("status")
            .arg(Arg::with_name("PATH")
                .short("f")
                .takes_value(true)
                .help("path of the directory from which the git governance will start analysing the git repos")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("status") {
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
        Ok(repo) => git::process_git_repo(repo, path),
        Err(err) => println!("Error: {} in opening git repository {:#?}", err, path)
    }
}