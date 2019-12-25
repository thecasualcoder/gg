use std::error::Error;
use std::process;

use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{Error as GitError, Repository, StatusOptions};
use regex::Regex;
use walkdir::DirEntry;

use crate::dir::DirectoryTreeOptions;
use crate::git::GitAction;
use crate::input_args::InputArgs;

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

pub fn
status(args: InputArgs, filter_list: Vec<Regex>) {
    let matches = args.get_matches();
    let filter_hidden = matches.is_present("traverse-hidden");

    let dir_tree_with_options = DirectoryTreeOptions {
        filter_list: filter_list,
        filter_hidden: filter_hidden,
    };

    let root_path = args.get_root_path("PATH");
    let root = root_path.to_str().expect(format!("{}", "Error in converting directory to string".red()).as_str());

    dir_tree_with_options.process_directories(root, process_directory).unwrap_or_else(|err| {
        println!("{} {}: {}", "Failed getting status for path".red(), root.red(), err);
        process::exit(1);
    });
}

fn process_directory(dir: &DirEntry) -> Result<(), Box<dyn Error>> {
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
                    statuses_in_dir.push("new files".to_string());
                };
                if git2::Status::is_wt_deleted(status) {
                    statuses_in_dir.push("deletions".to_string());
                };
                if git2::Status::is_wt_renamed(status) {
                    statuses_in_dir.push("renames".to_string());
                };
                if git2::Status::is_wt_typechange(status) {
                    statuses_in_dir.push("typechanges".to_string());
                };
                if git2::Status::is_wt_modified(status) {
                    statuses_in_dir.push("modifications".to_string());
                };
            };

//      Adapted from @Kurt-Bonatz in https://github.com/rust-lang/git2-rs/issues/332#issuecomment-408453956
        if self.repo.revparse_single("HEAD").is_ok() {
            let head_ref = self.repo.revparse_single("HEAD").expect("HEAD not found").id();
            let (is_ahead, is_behind) = self.repo.revparse_ext("@{u}")
                .ok()
                .and_then(|(upstream, _)| self.repo.graph_ahead_behind(head_ref, upstream.id()).ok())
                .unwrap_or((0, 0));

            if is_ahead > 0 {
                let push_string = format!("{} ahead", is_ahead);
                let push_string_colored = format!("{}", push_string.blue());
                statuses_in_dir.push(push_string_colored);
            }

            if is_behind > 0 {
                let pull_string = format!("{} behind", is_behind);
                let pull_string_colored = format!("{}", pull_string.yellow());
                statuses_in_dir.push(pull_string_colored);
            }
        }

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
