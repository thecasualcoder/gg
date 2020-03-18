use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{Error as GitError, Repository, StatusOptions};
use regex::Regex;
use std::path::PathBuf;

use crate::dir::DirectoryTreeOptions;
use crate::git::GitAction;
use crate::input_args::InputArgs;
use crate::progress::{ProgressReporter, ProgressTracker};

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("status")
        .arg(
            Arg::with_name("PATH")
                .short("f")
                .takes_value(true)
                .help("path at which to create the local repo"),
        )
        .arg(
            Arg::with_name("traverse-hidden")
                .short("i")
                .help("traverse through hidden directories also"),
        )
}

pub fn status(args: InputArgs, filter_list: Vec<Regex>) {
    let matches = args.get_matches();
    let filter_hidden = matches.is_present("traverse-hidden");

    let dir_tree_with_options = DirectoryTreeOptions {
        filter_list,
        filter_hidden,
    };

    let root_path = args.get_root_path("PATH");
    let root = root_path
        .to_str()
        .expect(format!("{}", "Error in converting directory to string".red()).as_str());

    let multi_bars = ProgressTracker::new(matches.value_of("jobs").and_then(|e| e.parse().ok()));
    dir_tree_with_options
        .process_directories(root)
        .flat_map(|dir| {
            dir.ok().and_then(|d| {
                if d.file_name().eq(".git") {
                    d.path().parent().map(|e| e.to_path_buf())
                } else {
                    None
                }
            })
        })
        .map(|dir| GitStatus { dir })
        .for_each(|status| multi_bars.start_task(status));
    multi_bars.join().unwrap();
}

pub struct GitStatus {
    dir: PathBuf,
}

impl<'a> GitAction for GitStatus {
    fn get_name(&self) -> String {
        self.dir.to_string_lossy().to_string()
    }

    fn git_action(&mut self, _progress: &ProgressReporter) -> Result<String, GitError> {
        let mut opts = StatusOptions::new();
        opts.include_ignored(true)
            .include_untracked(true)
            .recurse_untracked_dirs(false)
            .exclude_submodules(false);

        let repo = Repository::open(self.dir.clone())?;

        let git_statuses = repo.statuses(Some(&mut opts))?;
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
        }

        //      Adapted from @Kurt-Bonatz in https://github.com/rust-lang/git2-rs/issues/332#issuecomment-408453956
        if repo.revparse_single("HEAD").is_ok() {
            let head_ref = repo.revparse_single("HEAD").expect("HEAD not found").id();
            let (is_ahead, is_behind) = repo
                .revparse_ext("@{u}")
                .ok()
                .and_then(|(upstream, _)| repo.graph_ahead_behind(head_ref, upstream.id()).ok())
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

        Ok(if statuses_in_dir.is_empty() {
            "no changes".green().to_string()
        } else {
            statuses_in_dir.sort();
            statuses_in_dir.dedup();
            statuses_in_dir.join(", ").red().to_string()
        })
    }
}
