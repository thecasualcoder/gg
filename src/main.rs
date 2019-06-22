use walkdir::WalkDir;
use git2::{Repository, StatusOptions};
use std::path::{PathBuf, Path};
use colored::*;

fn main() {
    let mut all_git_dirs = vec![];

    for entry in WalkDir::new("/Users/ninanjohn/Trials")
        .follow_links(true)
        .contents_first(true)
        .same_file_system(true)
        {
            let entry = entry.unwrap();
            if entry.file_name().eq(".git") {
                all_git_dirs.push(entry.clone());
                let path = Path::new(entry.path().parent().unwrap());
                let mut opts = StatusOptions::new();
                let repo = Repository::open(path);
                let repo = repo.unwrap();
                if repo.is_bare() {
                    println!("{:#?}: {}", entry.path().parent().unwrap(), "bare".yellow());
                    continue;
                }

                opts.include_ignored(true)
                    .include_untracked(true)
                    .recurse_untracked_dirs(false)
                    .exclude_submodules(false);

                match repo.statuses(Some(&mut opts)) {
                    Ok(s) => {
                        // Todo: Filter the statuses to find untracked, new, deleteions, renames...
                        // Todo: https://github.com/rust-lang/git2-rs/blob/master/examples/status.rs
                        // Todo: Line 150
                        // Currently this does not filter anything.
                        println!("{:#?}: {}", entry.path().parent().unwrap(), "untracked".red());
                    }
                    Err(e) => {
                        panic!("Error in getting status for entry: {:#?}", entry.clone().path().parent().unwrap())
                    }
                }
                continue;
            }
        }
}
