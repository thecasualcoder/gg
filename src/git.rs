use git2::{Statuses, Error, StatusOptions};

pub trait GitAction {
    fn git_status(&self, opts: &mut StatusOptions) -> Result<Statuses, Error>;
}
