use std::path::Path;

use clap::{App, Arg, SubCommand};
use colored::*;
use git2::build::RepoBuilder;
use git2::Error as GitError;
use serde::{Deserialize, Serialize};

use crate::git::GitAction;
use crate::input_args::InputArgs;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitRepo {
    #[serde(alias = "remoteURL")]
    #[serde(rename = "remoteURL")]
    remote_url: String,
    #[serde(alias = "localPath")]
    #[serde(rename = "localPath")]
    local_path: String,
}

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("clone")
        .arg(Arg::with_name("local_path")
            .short("l")
            .takes_value(true)
            .default_value(".")
            .help("path at which to create the local repo. Defaults to '.'"))
        .arg(Arg::with_name("repo_url")
            .short("r")
            .takes_value(true)
            .multiple(true)
            .help("the remote git repo url"))
}

// gg clone -r url1 -r url2 -l local_root_path
// In addition to arguments passed, also check conf file. If 2 entries conflict from file and passed entry, use the passed entry to resolve the conflict.
// Arguments can have only one local path at which to clone. If user wishes multiple paths, they have to use a config file.
// Todo: test conflicting entries from arguments and file.
pub fn clone(args: InputArgs, mut clone_repos: Vec<GitRepo>) {
    let matches = args.get_matches();

    let remotes = matches.values_of("repo_url");
    let mut remote_urls: Vec<&str> = vec![];
    if remotes.is_some() {
        remote_urls = remotes.expect("failed parsing remote_urls from user").collect();
    }
//    let remote_urls: Vec<&str> = matches.values_of("repo_url").expect("failed getting remote_urls from user").collect();
    let local_path = matches.value_of("local_path").expect("failed parsing local path from arguments");

    let mut remotes_from_args: Vec<GitRepo> = vec![];
    for remote in remote_urls {
        let repo = GitRepo {
            remote_url: remote.to_string(),
            local_path: local_path.to_string(),
        };
        remotes_from_args.push(repo);
    }

    remotes_from_args.append(&mut clone_repos);

    for remote in remotes_from_args {
        let mut clone = GitClone { remote_url: remote.remote_url.as_str(), local_path: Path::new(remote.local_path.as_str()) };
        clone.git_action().expect(format!("Failed to clone repo {}, ", remote.remote_url).as_str());
    }
}

pub struct GitClone<'a> {
    pub remote_url: &'a str,
    pub local_path: &'a Path,
}

// Todo: Add spinner to show progress.
impl<'a> GitAction for GitClone<'a> {
    fn git_action(&mut self) -> Result<(), GitError> {
        RepoBuilder::new()
            .clone(self.remote_url, self.local_path)?;
        println!("{} - {} {} {:#?}", "Remote repo".green(), self.remote_url.blue(), "cloned locally at".green(),
                 self.local_path.as_os_str());

        Ok(())
    }
}