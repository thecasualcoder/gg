use git2::Error;

pub trait GitAction {
    fn git_action(&mut self) -> Result<(), Error>;
}
