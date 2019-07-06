use clap::{ArgMatches, SubCommand, Arg, App};
use git2::{Repository, StatusOptions, Statuses, Error};
use std::env::current_dir;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use colored::*;
use crate::git::GitAction;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("status")
        .arg(Arg::with_name("PATH")
            .short("f")
            .takes_value(true)
            .help("path at which to create the local repo")
        )
}

pub fn status(matches: &ArgMatches) {
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
            match git_status(repo) {
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

fn git_status<'a>(repo: Repository) -> Result<Vec<&'a str>, String> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(true)
        .include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(false);

    let gst = GitStatus { repo: repo };
    let git_statuses = gst.git_status(&mut opts);
    match git_statuses {
        Ok(statuses) => {
            let mut statuses_in_dir = vec![];
            for entry in statuses
                .iter()
                .filter(|e| e.status() != git2::Status::CURRENT)
                {
                    let status = &entry.status();
                    if git2::Status::is_wt_new(status) {
                        statuses_in_dir.push("new files");
                    };
                    if git2::Status::is_wt_deleted(status) {
                        statuses_in_dir.push("deletions");
                    };
                    if git2::Status::is_wt_renamed(status) {
                        statuses_in_dir.push("renames");
                    };
                    if git2::Status::is_wt_typechange(status) {
                        statuses_in_dir.push("typechanges");
                    };
                    if git2::Status::is_wt_modified(status) {
                        statuses_in_dir.push("modifications");
                    };
                };
            return Ok(statuses_in_dir);
        }
        Err(e) => {
            return Err(e.to_string());
        }
    }
}

pub struct GitStatus {
    repo: Repository,
}

impl GitAction for GitStatus {
    fn git_status(&self, opts: &mut StatusOptions) -> Result<Statuses, Error> {
        self.repo.statuses(Some(opts))
    }
}