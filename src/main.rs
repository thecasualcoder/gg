use std::env::current_dir;
use std::path::{Path, PathBuf};

use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{Repository, StatusOptions};
use walkdir::WalkDir;

fn main() {
    let matches = App::new("Git Governance")
        .version("1.0")
        .subcommand(SubCommand::with_name("status")
            .arg(Arg::with_name("PATH")
                .short("f")
                .takes_value(true)
                .help("path of the directory from which the git governance will start analysing the git repos")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("status") {
        let mut path = current_dir().unwrap();
        if let Some(option) = matches.value_of("PATH") {
            path = PathBuf::from(option)
        };
        let directory = WalkDir::new(path.to_str().unwrap());
        let mut all_git_dirs = vec![];

        for entry in directory
            .follow_links(true)
            .contents_first(true)
            .same_file_system(true)
            {
                let entry = entry.unwrap();
                if entry.file_name().eq(".git") {
                    all_git_dirs.push(entry.clone());
                    let path = Path::new(entry.path().parent().unwrap());
                    let mut opts = StatusOptions::new();
                    let repo = Repository::open(path);
                    let repo = repo.unwrap();
                    if repo.is_bare() {
                        println!("{:#?}: {}", entry.path().parent().unwrap(), "bare".yellow());
                        continue;
                    };

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
                                println!("{:#?}: {}", entry.path().parent().unwrap(), "no changes".green());
                            } else {
                                statuses_in_dir.sort();
                                statuses_in_dir.dedup();
                                println!("{:#?}: {}", entry.path().parent().unwrap(), statuses_in_dir.join(", ").red());
                            };
                        }
                        Err(e) => {
                            panic!("Error in getting status for entry: {:#?}", entry.clone().path().parent().unwrap());
                        }
                    };
                    continue;
                };
            }
    }
}
