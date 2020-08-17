use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{Error as GitError, Repository};
use git2::BranchType::Local;

use crate::input_args::InputArgs;
use std::process;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("branches")
        .arg(
            Arg::with_name("repo_path")
                .short("r")
                .default_value(".")
                .takes_value(true)
                .help("path of the repo where branches are to be compared. Defaults to '.'"),
        )
        .arg(
            Arg::with_name("main_branch")
                .short("b")
                .default_value("origin/master")
                .takes_value(true)
                .help("the main branch against which other local branches are to be compared. Defaults to 'origin/master'"),
        )
}

pub fn branches(args: InputArgs) {
    let root_path = args.get_root_path("repo_path");
    let matches = args.get_matches();
    let main_branch = matches.value_of("main_branch").unwrap();
    let repo = Repository::open(root_path).expect("Failed to open git repo");

    let mut git_branches = GitBranches {
        repo: repo,
        main_branch: String::from(main_branch),
    };

    let branches = git_branches.get_branches().unwrap_or_else(|err| {
        println!("{} {}", "Failed to get local branches. Err:".red(), err);
        process::exit(1);
    });

    branches
        .into_iter()
        .for_each(|branch| {
            if branch == "master" {
                return;
            }
            git_branches.compare_to_main_branch(branch.as_str())
        })
}

pub struct GitBranches {
    repo: Repository,
    main_branch: String,
}

impl GitBranches {
    fn get_branches(&mut self) -> Result<Vec<String>, GitError> {
        let local_branches = self.repo.branches(Some(Local))?;
        let mut branch_vec = vec![];
        local_branches.for_each(|br| {
            let branch_ref = br.expect("Failed to get branch");
            let branch_name = branch_ref.0.name().expect("Failed to get valid name of branch")
                .expect("Failed to convert branch name to string");
            branch_vec.push(branch_name.to_string())
        });
        Ok(branch_vec)
    }

    fn compare_to_main_branch(&mut self, branch: &str) {
        let mut branch_status = vec![];
        self.repo.revparse_single(branch).map_or_else(|_err| {
            print!("{} {} {}", "No such branch".red(), branch.red(), "found".red());
        }, |reference|{
            let head_ref = reference.id();
            let (is_ahead, is_behind) = self.repo
                .revparse_ext(self.main_branch.as_str())
                .ok()
                .and_then(|(upstream, _)| self.repo.graph_ahead_behind(head_ref, upstream.id()).ok())
                .unwrap_or((0, 0));

            if is_ahead > 0 {
                let ahead = format!("{} ahead", is_ahead);
                let ahead_string_colored = format!("{}", ahead.green());
                branch_status.push(ahead_string_colored);
            }

            if is_behind > 0 {
                let behind = format!("{} behind", is_behind);
                let behind_string_colored = format!("{}", behind.yellow());
                branch_status.push(behind_string_colored);
            }
            println!("{} {}: {}", "Branch".blue(), branch.blue(), branch_status.join(", "));
        });
    }
}