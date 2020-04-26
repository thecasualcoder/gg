use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{
    AutotagOption, Error as GitError, FetchOptions, RemoteCallbacks,
    Repository,
};
use regex::Regex;
use std::path::PathBuf;

use crate::dir::DirectoryTreeOptions;
use crate::git::GitAction;
use crate::input_args::InputArgs;
use crate::progress::{ProgressReporter, ProgressTracker};
use crate::conf::*;

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

pub fn fetch(args: InputArgs, filter_list: Vec<Regex>) {
    let matches = args.get_matches();
    let filter_hidden = matches.is_present("traverse-hidden");

    let dir_tree_with_options = DirectoryTreeOptions {
        filter_list: filter_list,
        filter_hidden: filter_hidden,
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
        .map(|dir| GitFetch {
            dir,
            remote: "origin".to_string(),
        })
        .for_each(|clone| multi_bars.start_task(clone));

    multi_bars.join().unwrap();
}

pub struct GitFetch {
    dir: PathBuf,
    remote: String,
}

impl<'a> GitAction for GitFetch {
    fn get_name(&self) -> String {
        format!("{} from {:?}", self.remote, self.dir)
    }

    fn git_action(&mut self, prog: &ProgressReporter) -> Result<String, GitError> {
        let repo = Repository::open(self.dir.clone())?;
        let path = self.dir.parent();
        let remotes = repo.remotes()?;

        let mut remote = if remotes.iter().any(|remote| remote == Some(&self.remote)) {
            repo.find_remote(&self.remote)
                .or_else(|_| repo.remote_anonymous(&self.remote))?
        } else {
            // TODO Use proper error handling
            return Ok(format!("{} {:#?}", "remote named {} not found for".red(), path));
        };
        let mut cb = RemoteCallbacks::new();
        cb.credentials(ssh_auth_callback);
        cb.transfer_progress(prog.get_callback());

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(cb);
        Ok(match remote.download(&[], Some(&mut fetch_options)) {
            Err(e) => format!("{} {}", "Failed with error:".red(), e.message()),
            Ok(_) => {
                let stats = remote.stats();
                let res = if stats.local_objects() > 0 {
                    format!(
                        "{} {}/{} {} {} {} {} {}",
                        "Received".green(),
                        stats.indexed_objects(),
                        stats.total_objects(),
                        "objects in".green(),
                        stats.received_bytes(),
                        " bytes (used ".green(),
                        stats.local_objects(),
                        "local objects)".green()
                    )
                } else {
                    format!(
                        "{} {}/{} {} {} {}",
                        "Received".green(),
                        stats.indexed_objects(),
                        stats.total_objects(),
                        "objects in".green(),
                        stats.received_bytes(),
                        "bytes".green()
                    )
                };
                remote.disconnect();
                remote.update_tips(None, true, AutotagOption::Unspecified, None)?;

                res
            }
        })
    }
}
