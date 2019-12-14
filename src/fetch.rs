use std::env::current_dir;
use std::error::Error;
use std::process;

use clap::{App, Arg, ArgMatches, SubCommand};
use colored::*;
use git2::{AutotagOption, Cred, CredentialType, Error as GitError, FetchOptions, Remote,
           RemoteCallbacks, Repository};
use walkdir::{DirEntry};

use regex::Regex;

use crate::dir::DirectoryTreeOptions;
use crate::git::GitAction;
use std::path::Path;


pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("fetch")
        .arg(Arg::with_name("PATH")
            .short("f")
            .takes_value(true)
            .help("path at which to fetch the git repos"))
        .arg(Arg::with_name("traverse-hidden")
            .short("i")
            .help("traverse through hidden directories also")
        )
}

pub fn fetch(args: &ArgMatches, filter_list: Vec<Regex>) {
    let filter_hidden = args.is_present("traverse-hidden");

    let root_path = match args.value_of("PATH") {
        Some(path) => { Path::new(path).to_path_buf() }
        None => {
            current_dir().unwrap_or_else(|err| {
                println!("{} {}", "Error accessing current_dir:".red(), err);
                process::exit(1);
            })
        }
    };

    let root = root_path.to_str().expect(format!("{}", "Error in converting directory to string".red()).as_str());

    let dir_tree_with_options = DirectoryTreeOptions {
        filter_list: filter_list,
        filter_hidden: filter_hidden,
    };

    dir_tree_with_options.process_directories(root, process_directory).unwrap_or_else(|err| {
        println!("{} {}: {}", "Failed fetching for".red(), root.red(), err);
        process::exit(1);
    });
}

fn process_directory(dir: &DirEntry) -> Result<(), Box<dyn Error>> {
    if dir.file_name().eq(".git") {
        match dir.path().parent() {
            Some(dir) => {
                let repo = Repository::open(dir)?;
                fetch_repo(repo)?;
            }
            None => {
                println!("{} {:#?}", "error accessing parent directory of".red(), dir.path())
            }
        }
    }
    Ok(())
}

fn fetch_repo<'a>(repo: Repository) -> Result<(), Box<dyn Error>> {
    let path = repo.path().parent().expect("Failed to get path from repo");
    let remotes = repo.remotes()?;

    // TODO: handle all remotes
    if remotes.iter().any(|remote| remote == Some("origin")) {
        let remote = "origin";
        print!("{} {} {} {:#?} -> ", "\nFetching".blue(), remote.blue(), "for".blue(), path);

        let mut cb = RemoteCallbacks::new();

        let remote = repo
            .find_remote(remote)
            .or_else(|_| repo.remote_anonymous(remote))?;

        cb.credentials(git_credentials_callback);

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);

        let mut fetch = GitFetch { remote: remote, fetch_options: fo };
        fetch.git_action()?
    } else {
        println!("{} {:#?}", "remote named origin not found for".red(), path);
    }
    return Ok(());
}

pub fn git_credentials_callback(_user: &str, _user_from_url: Option<&str>, _cred: CredentialType)
                                -> Result<Cred, GitError> {
    match std::env::var("HOME") {
        Ok(home) => {
            let path = format!("{}/.ssh/id_rsa", home);
            let credentials_path = std::path::Path::new(&path);
            match credentials_path.exists() {
                true => Cred::ssh_key("git", None, credentials_path, None),
                false => Err(GitError::from_str(&format!("unable to get key from {}", path))),
            }
        }
        Err(_) => Err(GitError::from_str("unable to get env variable HOME")),
    }
}

pub struct GitFetch<'a> {
    remote: Remote<'a>,
    fetch_options: FetchOptions<'a>,
}

impl<'a> GitAction for GitFetch<'a> {
    fn git_action(&mut self) -> Result<(), GitError> {
        match self.remote.download(&[], Some(&mut self.fetch_options)) {
            Err(e) => {
                print!("{} {}", "Failed with error:".red(), e.message());
            }
            Ok(_) => {
                let stats = self.remote.stats();
                if stats.local_objects() > 0 {
                    print!("{} {}/{} {} {} {} {} {}",
                           "Received".green(),
                           stats.indexed_objects(),
                           stats.total_objects(),
                           "objects in".green(),
                           stats.received_bytes(),
                           " bytes (used ".green(),
                           stats.local_objects(),
                           "local objects)".green()
                    );
                } else {
                    print!(
                        "{} {}/{} {} {} {}",
                        "Received".green(),
                        stats.indexed_objects(),
                        stats.total_objects(),
                        "objects in".green(),
                        stats.received_bytes(),
                        "bytes".green()
                    );
                }
                self.remote.disconnect();
                self.remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
            }
        }
        Ok(())
    }
}


