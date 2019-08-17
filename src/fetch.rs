use std::env::current_dir;
use std::error::Error;

use clap::{App, Arg, ArgMatches, SubCommand};
use git2::{Error as GitError, Cred, CredentialType, AutotagOption, FetchOptions, Remote,
           RemoteCallbacks, Repository};
use walkdir::{DirEntry, WalkDir};

use crate::git::GitAction;

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
        Some(path) => {
            match process_directories(path) {
                Ok(_) => {}
                Err(err) => panic!("Error: {}", err),
            }
        }
        None => {
            match current_dir() {
                Ok(dir) => {
                    match dir.to_str() {
                        Some(dir) => {
                            match process_directories(dir) {
                                Ok(_) => {}
                                Err(err) => panic!("Error: {}", err),
                            }
                        }
                        None => panic!("Error in converting current directory to string")
                    }
                }
                Err(err) => panic!("Error: {}", err),
            }
        }
    };
}

fn process_directories(path: &str) -> Result<(), Box<dyn Error>> {
    let directory = WalkDir::new(path);

    for entry in directory
        .follow_links(false)
        .contents_first(true)
        .same_file_system(true)
        {
            process_directory(entry?)?
        }
    Ok(())
}

fn process_directory(dir: DirEntry) -> Result<(), Box<dyn Error>> {
    if dir.file_name().eq(".git") {
        match dir.path().parent() {
            Some(dir) => {
                let repo = Repository::open(dir)?;
                fetch_repo(repo)?;
            }
            None => {}
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
        print!("\nFetching {} for {:#?} -> ", remote, path);

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
        println!("remote named origin not found for {:#?}", path);
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
                print!("Failed with error: {}", e.message())
            }
            Ok(_) => {
                let stats = self.remote.stats();
                if stats.local_objects() > 0 {
                    print!(
                        "Received {}/{} objects in {} bytes (used {} local \
                 objects)",
                        stats.indexed_objects(),
                        stats.total_objects(),
                        stats.received_bytes(),
                        stats.local_objects()
                    );
                } else {
                    print!(
                        "Received {}/{} objects in {} bytes",
                        stats.indexed_objects(),
                        stats.total_objects(),
                        stats.received_bytes()
                    );
                }
                self.remote.disconnect();
                self.remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
            }
        }
        Ok(())
    }
}


