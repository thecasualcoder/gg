use std::env::current_dir;
use std::path::Path;

use clap::{App, AppSettings, Arg, SubCommand};
use colored::*;
use git2::{Repository, StatusOptions};
use walkdir::{WalkDir, DirEntry};

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
        Ok(repo) => process_git_repo(repo, path),
        Err(err) => println!("Error: {} in opening git repository {:#?}", err, path)
    }
}

fn process_git_repo(repo: Repository, path: &Path) {
    if repo.is_bare() {
        println!("{:#?}: {}", path, "bare".yellow());
        return;
    };

    let mut opts = StatusOptions::new();
    opts.include_ignored(true)
        .include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(false);

    match repo.statuses(Some(&mut opts)) {
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
            if statuses_in_dir.is_empty() {
                println!("{:#?}: {}", path, "no changes".green());
            } else {
                statuses_in_dir.sort();
                statuses_in_dir.dedup();
                println!("{:#?}: {}", path, statuses_in_dir.join(", ").red());
            };
        }
        Err(e) => {
            panic!("Error: {} in getting status for dir: {:#?}", e, path);
        }
    };
}

