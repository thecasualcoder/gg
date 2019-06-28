use git2::{Repository, StatusOptions};

pub fn status<'a>(repo: Repository) -> Result<Vec<&'a str>, String> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(true)
        .include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(false);

    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => {
            let mut statuses_in_dir = vec![];
            for entry in statuses
                .iter()
                .filter(|e| e.status() != git2::Status::CURRENT)
                {
                    let status = &entry.status();
                    if git2::Status::is_wt_new(status) {
                        statuses_in_dir.push("new files");
                    };
                    if git2::Status::is_wt_deleted(status) {
                        statuses_in_dir.push("deletions");
                    };
                    if git2::Status::is_wt_renamed(status) {
                        statuses_in_dir.push("renames");
                    };
                    if git2::Status::is_wt_typechange(status) {
                        statuses_in_dir.push("typechanges");
                    };
                    if git2::Status::is_wt_modified(status) {
                        statuses_in_dir.push("modifications");
                    };
                };
            return Ok(statuses_in_dir);
        }
        Err(e) => {
            return Err(e.to_string());
        }
    };
}
