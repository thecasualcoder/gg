use std::path::PathBuf;

use git2::Error;

use crate::progress::ProgressReporter;

pub trait GitAction {
    fn git_action(&mut self, prog: &ProgressReporter) -> Result<String, Error>;
    fn get_name(&self) -> String;

    fn relative_path(dir: &PathBuf, root: &String) -> String {
        let relative_path = dir.to_string_lossy().to_string().replace(root, "");
        format!(".{}", relative_path)
    }

    fn do_git_action(&mut self, prog: ProgressReporter) {
        prog.start();
        match self.git_action(&prog) {
            Ok(res) => prog.finalize(&res),
            Err(err) => prog.abandon(err),
        }
    }
}
