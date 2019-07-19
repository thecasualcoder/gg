use std::env::current_dir;

use std::path::Path;
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::*;
use git2::{Error as GitError, Repository, Statuses, StatusOptions};
use walkdir::{DirEntry, WalkDir};

use crate::git::GitAction;
use std::error::Error;

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
                        None => panic!("Error in converting current directory to string")
                    }
                }
                Err(err) => panic!("Error: {}", err),
            }
        }
    };
}

fn process_directories(path: &str) -> Result<(), Box<dyn Error>> {
    let directory = WalkDir::new(path);

    for entry in directory
        .follow_links(true)
        .contents_first(true)
        .same_file_system(true)
        {
            process_directory(entry?)?
        }
    Ok(())
}

fn process_directory(dir: DirEntry) -> Result<(), Box<dyn Error>> {
    if dir.file_name().eq(".git") {
        match dir.path().parent() {
            Some(dir) => {
                let path = Repository::open(dir)?;
                let statuses = process_git_directory(path)?;
                print_repo_status(dir, statuses);
            }
            None => {}
        }
    }
    Ok(())
}

fn process_git_directory<'a>(repo: Repository) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(true)
        .include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(false);
    git_status(repo, &mut opts)
}

fn print_repo_status(path: &Path, statuses_in_dir: Vec<&str>) {
    let mut statuses_in_dir = statuses_in_dir;
    if statuses_in_dir.is_empty() {
        println!("{:#?}: {}", path, "no changes".green());
    } else {
        statuses_in_dir.sort();
        statuses_in_dir.dedup();
        println!("{:#?}: {}", path, statuses_in_dir.join(", ").red());
    };
}

fn git_status<'a>(repo: Repository, opts: &mut StatusOptions) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let gst = GitStatus { repo };
    let git_statuses = gst.git_status(opts)?;
    let mut statuses_in_dir = vec![];
    for entry in git_statuses
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

pub struct GitStatus {
    repo: Repository,
}

impl GitAction for GitStatus {
    fn git_status(&self, opts: &mut StatusOptions) -> Result<Statuses, GitError> {
        self.repo.statuses(Some(opts))
    }
}
