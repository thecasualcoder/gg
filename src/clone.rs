use std::path::PathBuf;

use clap::{App, Arg, SubCommand};
use colored::*;
use git2::build::RepoBuilder;
use git2::{Error as GitError, FetchOptions, RemoteCallbacks};
use serde::{Deserialize, Serialize};

use crate::conf;
use crate::git::GitAction;
use crate::input_args::InputArgs;
use crate::progress::{ProgressReporter, ProgressTracker};

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
        .arg(
            Arg::with_name("local_path")
                .short("l")
                .takes_value(true)
                .default_value(".")
                .help("path at which to create the local repo. Defaults to '.'"),
        )
        .arg(
            Arg::with_name("repo_url")
                .short("r")
                .takes_value(true)
                .multiple(true)
                .help("the remote git repo url"),
        )
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
        remote_urls = remotes
            .expect("failed parsing remote_urls from user")
            .collect();
        println!("{}", "Cloning remotes passed as arguments".blue());
    } else {
        println!("{}", "No remotes were passed as arguments.".yellow())
    }
    let local_path = matches
        .value_of("local_path")
        .expect("failed parsing local path from arguments");

    let mut remotes_from_args: Vec<GitRepo> = vec![];
    for remote in remote_urls {
        let remote_url_string = remote.to_string();
        let splits = remote_url_string
            .split("/")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .last()
            .expect("Failed to get repo name from remote URL")
            .split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let repo = GitRepo {
            remote_url: remote_url_string,
            local_path: format!("{}/{}", local_path.to_string(), splits[0]),
        };
        remotes_from_args.push(repo);
    }

    if clone_repos.is_empty() {
        println!("{}", "No remotes configured in conf file".yellow())
    } else {
        println!("{}", "Cloning remotes configured in conf file".blue());
        remotes_from_args.append(&mut clone_repos);
    }

    if remotes_from_args.is_empty() {
        println!("{}", "Please configure conf file to clone repositories or pass the necessary values as arguments".blue())
    }

    let multi_bars = ProgressTracker::new(matches.value_of("jobs").and_then(|e| e.parse().ok()));
    remotes_from_args
        .into_iter()
        .map(|remote| GitClone {
            use_ssh: remote.remote_url.contains("git@"),
            remote_url: remote.remote_url,
            local_path: remote.local_path.into(),
        })
        .for_each(|clone| multi_bars.start_task(clone));

    multi_bars.join().unwrap();
}

pub struct GitClone {
    pub remote_url: String,
    pub local_path: PathBuf,
    pub use_ssh: bool,
}

impl GitAction for GitClone {
    fn get_name(&self) -> String {
        self.remote_url.clone()
    }

    fn git_action(&mut self, prog: &ProgressReporter) -> Result<String, GitError> {
        let mut builder = RepoBuilder::new();

        let mut fetch_options = FetchOptions::new();
        let mut callback = RemoteCallbacks::new();

        if self.use_ssh {
            callback.credentials(conf::ssh_auth_callback);
        }

        callback.transfer_progress(prog.get_callback());

        fetch_options.remote_callbacks(callback);
        builder.fetch_options(fetch_options);

        builder.clone(&self.remote_url, &self.local_path)?;

        Ok(format!(
            "{} {} {:#?}",
            "Remote repo".green(),
            "cloned locally at".green(),
            self.local_path.as_os_str()
        ))
    }
}
