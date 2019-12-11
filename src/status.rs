use std::env::current_dir;
use std::error::Error;
use std::process;

use clap::{App, Arg, ArgMatches, SubCommand};
use colored::*;
use git2::{Error as GitError, Repository, StatusOptions};
use walkdir::{DirEntry, WalkDir};

use crate::git::GitAction;
use crate::dir::DirectoryTreeOptions;
use regex::Regex;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("status")
        .arg(Arg::with_name("PATH")
            .short("f")
            .takes_value(true)
            .help("path at which to create the local repo"))
        .arg(Arg::with_name("traverse-hidden")
            .short("i")
            .help("traverse through hidden directories also")
        )
}

pub fn status(matches: &ArgMatches, filter_list: Vec<Regex>) {
    let filter_hidden = matches.is_present("traverse-hidden");
    println!("filter_hidden: {}", filter_hidden);

    match matches.value_of("PATH") {
        Some(path) => process_directories(path, filter_list, filter_hidden).unwrap_or_else(|err| {
            println!("{} {}: {}", "Failed getting status for path".red(), path.red(), err);
            process::exit(1);
        }),
        None => {
            match current_dir() {
                Ok(dir) => {
                    match dir.to_str() {
                        Some(dir) => process_directories(dir, filter_list, filter_hidden).unwrap_or_else(|err| {
                            println!("{} {}", "Failed to get status for current directory: ".red(), err);
                            process::exit(1);
                        }),
                        None => {
                            println!("{}", "Error in converting current directory to string".red());
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    println!("{} {}", "Error accessing current_dir:".red(), err);
                    process::exit(1);
                }
            }
        }
    };
}

fn process_directories(path: &str, filter_list: Vec<Regex>, filter_hidden: bool) -> Result<(), Box<dyn Error>> {
    let dir_tree_with_options = DirectoryTreeOptions {
        filter_list: filter_list,
        filter_hidden: filter_hidden,
    };

    let tree = WalkDir::new(path)
        .follow_links(false)
        .contents_first(false)
        .same_file_system(true)
        .into_iter()
        .filter_entry(|e| {
            dir_tree_with_options.should_filter(e).expect(format!("failed to filter for entry {:#?}", e).as_str())
        });

    for entry in tree {
//        println!("{:#?}", entry)
        process_directory(entry?)?
    }
    Ok(())
}

fn process_directory(dir: DirEntry) -> Result<(), Box<dyn Error>> {
    if dir.file_name().eq(".git") {
        match dir.path().parent() {
            Some(dir) => {
                let repo = Repository::open(dir)?;
                let mut opts = StatusOptions::new();
                opts.include_ignored(true)
                    .include_untracked(true)
                    .recurse_untracked_dirs(false)
                    .exclude_submodules(false);
                let mut gst = GitStatus { repo: repo, opts: &mut opts };
                gst.git_action()?
            }
            None => {
                println!("{} {:#?}", "error accessing parent directory of".red(), dir.path())
            }
        }
    }
    Ok(())
}

pub struct GitStatus<'a> {
    repo: Repository,
    opts: &'a mut StatusOptions,
}

impl<'a> GitAction for GitStatus<'a> {
    fn git_action(&mut self) -> Result<(), GitError> {
        let git_statuses = self.repo.statuses(Some(self.opts))?;
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

        if statuses_in_dir.is_empty() {
            println!("{:?}: {}", self.repo.path().parent().expect("Failed to get path from repository"),
                     "no changes".green());
        } else {
            statuses_in_dir.sort();
            statuses_in_dir.dedup();
            println!("{:?}: {}", self.repo.path().parent().expect("Failed to get path from repository"),
                     statuses_in_dir.join(", ").red());
        }
        Ok(())
    }
}
