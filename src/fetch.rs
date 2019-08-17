use std::env::current_dir;
use std::error::Error;

use clap::{App, Arg, ArgMatches, SubCommand};
use git2::{AutotagOption, FetchOptions, RemoteCallbacks, Repository};
use walkdir::{DirEntry, WalkDir};

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
                let path = Repository::open(dir)?;
                fetch_repo(path, dir.to_str())?;
            }
            None => {}
        }
    }
    Ok(())
}

fn fetch_repo<'a>(repo: Repository, directory_name: Option<&str>) -> Result<(), Box<dyn Error>> {
    let remotes = repo.remotes()?;
    // TODO: handle all remotes
    if remotes.iter().any(|remote| remote == Some("origin")) {
        let remote = "origin";

        // Figure out whether it's a named remote or a URL
        print!("\nFetching {} for {} -> ", remote, directory_name.unwrap());
        let mut cb = RemoteCallbacks::new();
        let mut remote = repo
            .find_remote(remote)
            .or_else(|_| repo.remote_anonymous(remote))?;

        cb.credentials(git_credentials_callback);

        // Download the packfile and index it. This function updates the amount of
        // received data and the indexer stats which lets you inform the user about
        // progress.
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        match remote.download(&[], Some(&mut fo)) {
            Err(e) => {
                print!("Failed with error: {}", e.message())
            }
            Ok(_) => {
                // If there are local objects (we got a thin pack), then tell the user
                // how many objects we saved from having to cross the network.
                let stats = remote.stats();
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
                // Disconnect the underlying connection to prevent from idling.
                remote.disconnect();


                // Update the references in the remote's namespace to point to the right
                // commits. This may be needed even if there was no packfile to download,
                // which can happen e.g. when the branches have been changed but all the
                // needed objects are available locally.
                remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
            }
        }
    } else {
        println!("remote named origin not found for {}", directory_name.unwrap());
    }
    return Ok(());
}

pub fn git_credentials_callback(_user: &str, _user_from_url: Option<&str>, _cred: git2::CredentialType)
                                -> Result<git2::Cred, git2::Error> {
    match std::env::var("HOME") {
        Ok(home) => {
            let path = format!("{}/.ssh/id_rsa", home);
            let credentials_path = std::path::Path::new(&path);
            match credentials_path.exists() {
                true => git2::Cred::ssh_key("git", None, credentials_path, None),
                false => Err(git2::Error::from_str(&format!("unable to get key from {}", path))),
            }
        }
        Err(_) => Err(git2::Error::from_str("unable to get env variable HOME")),
    }
}


