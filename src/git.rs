use git2::{Error, Statuses};

pub trait GitAction {
    fn git_action(&mut self) -> Result<Statuses, Error>;
}
