use git2::{Error, Statuses, StatusOptions};

pub trait GitAction {
    fn git_action(&mut self) -> Result<Statuses, Error>;
}
