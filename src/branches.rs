use std::path::PathBuf;

use clap::{App, Arg, SubCommand};
use git2::{Error as GitError, Repository};
use git2::BranchType::Local;

use crate::input_args::InputArgs;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("branches")
        .arg(
            Arg::with_name("repo_path")
                .short("r")
                .default_value(".")
                .takes_value(true)
                .help("path of the repo where branches are to be compared. Defaults to '.'"),
        )
}

pub fn branches(args: InputArgs) {
    let root_path = args.get_root_path("repo_path");

    // Todo: Use this to do the comparisons
    let mut branches = GitBranches {
        local_path: root_path,
        main_branch: String::from("master"),
    };

    let _branches = branches.get_branches();
}

pub struct GitBranches {
    local_path: PathBuf,
    main_branch: String,
}

impl GitBranches {
    fn get_branches(&mut self) -> Result<Vec<String>, GitError> {
        let repo = Repository::open(self.local_path.clone())?;
        let local_branches = repo.branches(Some(Local))?;
        local_branches.for_each(|br| {
            let branch_ref = br.expect("Failed to get branch");
            let branch_name = branch_ref.0.name().expect("Failed to get valid name of branch").unwrap_or("");
            println!("{}", branch_name)
        });
        Ok(vec![])
    }
}