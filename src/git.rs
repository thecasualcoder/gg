use git2::{Error, Statuses, StatusOptions};

pub trait GitAction {
    fn git_status(&self, opts: &mut StatusOptions) -> Result<Statuses, Error>;
}
