use std::env::current_dir;
use std::error::Error;
use std::process;

use clap::{App, Arg, ArgMatches, SubCommand};
use colored::*;
use git2::{AutotagOption, Cred, CredentialType, Error as GitError, FetchOptions, Remote,
           RemoteCallbacks, Repository};
use walkdir::{DirEntry, WalkDir};

use crate::git::GitAction;
use std::path::Path;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("fetch")
        .arg(Arg::with_name("PATH")
            .short("f")
            .takes_value(true)
            .help("path at which to fetch the git repos")
        )
}

pub fn fetch(matches: &ArgMatches) {
    match matches.value_of("PATH") {
        Some(path) => process_directories(path).unwrap_or_else(|err| {
            println!("{} {}: {}", "Failed fetching for".red(), path.red(), err);
            process::exit(1);
        }),
        None => {
            match current_dir() {
                Ok(dir) => {
                    match dir.to_str() {
                        Some(dir) => process_directories(dir).unwrap_or_else(|err| {
                            println!("{} {}", "Failed fetching for".red(), err);
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

fn process_directories(path: &str) -> Result<(), Box<dyn Error>> {
    let tree = WalkDir::new(path)
        .follow_links(false)
        .contents_first(true)
        .same_file_system(true)
        .into_iter()
        .filter_entry(|e| check_filter(e));

    for entry in tree {
//        println!("{:#?}", entry?.file_name());
        process_directory(entry?)?
    }
    Ok(())
}

fn check_filter(entry: &DirEntry) -> bool {
    entry
        .path()
        .to_str()
        .map(|s| is_dir(s))
        .unwrap_or(false)
}

fn is_dir(path: &str) -> bool {
    let path = Path::new(path);
    return path.is_dir();
}
fn process_directory(dir: DirEntry) -> Result<(), Box<dyn Error>> {
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


