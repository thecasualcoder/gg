use crate::progress::ProgressReporter;
use git2::Error;

pub trait GitAction {
    fn git_action(&mut self, prog: &ProgressReporter) -> Result<String, Error>;
    fn get_name(&self) -> String;

    fn do_git_action(&mut self, prog: ProgressReporter) {
        match self.git_action(&prog) {
            Ok(res) => prog.finalize(&res),
            Err(err) => prog.abandon(err),
        }
    }
}
