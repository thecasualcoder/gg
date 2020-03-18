use colored::Colorize;
use git2::{Error, Progress};
use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};
use rayon::ThreadPool;

use crate::git::GitAction;

const STYLE_PRELOAD: &str = "{prefix:>40!.blue} {spinner} {wide_msg:.cyan}";
const STYLE_LOAD: &str = "{prefix:>40!.blue} {msg} {wide_bar} {percent:>3}% {eta}";
const STYLE_DONE: &str = "{prefix:>40!.blue} {wide_msg} {elapsed_precise}";
const STYLE_ERROR: &str = "{prefix:>40!.blue} {wide_msg:.red}";

pub enum ProgressTracker {
    MultiThread {
        progress: MultiProgress,
        pool: ThreadPool,
    },
    MonoThread,
}

impl ProgressTracker {
    pub fn new(threading_settings: Option<usize>) -> Self {
        match threading_settings {
            Some(1) => ProgressTracker::MonoThread,
            _ => {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threading_settings.unwrap_or(0))
                    .build()
                    .unwrap();
                let progress = MultiProgress::new();
                progress.set_move_cursor(true);
                ProgressTracker::MultiThread { progress, pool }
            }
        }
    }

    pub fn new_bar(&self, remote_url: &str) -> ProgressReporter {
        let mut prog_bar = ProgressBar::new_spinner()
            .with_style(ProgressStyle::default_bar().template(STYLE_PRELOAD));

        if let ProgressTracker::MultiThread { progress, .. } = self {
            prog_bar = progress.add(prog_bar);
        }

        prog_bar.set_prefix(&remote_url.blue());
        prog_bar.set_message(&"Waiting for process to begin".cyan());

        ProgressReporter(prog_bar)
    }

    pub fn start_task(&self, mut action: impl GitAction + Send + 'static) {
        let progress_bar = self.new_bar(&action.get_name());

        if let ProgressTracker::MultiThread { pool, .. } = self {
            pool.spawn(move || action.do_git_action(progress_bar));
        } else {
            action.do_git_action(progress_bar);
        }
    }

    pub fn join(self) -> Result<(), std::io::Error> {
        if let ProgressTracker::MultiThread { progress, .. } = self {
            progress.join()?;
        }

        Ok(())
    }
}

pub struct ProgressReporter(ProgressBar);

#[derive(PartialEq)]
enum Step {
    Waiting,
    Downloading,
    Unpacking,
}

impl ProgressReporter {
    pub fn finalize(self, status: &str) {
        self.0.set_draw_delta(0);
        self.0
            .set_style(ProgressStyle::default_bar().template(STYLE_DONE));
        self.0.finish_with_message(status);
    }

    pub fn abandon(self, err: Error) {
        self.0.set_draw_delta(0);
        self.0
            .set_style(ProgressStyle::default_bar().template(STYLE_ERROR));
        // self.0.println(format!("{}", err));
        self.0.abandon_with_message(err.message());
    }

    pub fn get_callback<'a>(&'a self) -> impl FnMut(Progress) -> bool + 'a {
        let prog_bar = self.0.clone();
        let mut step = Step::Waiting;

        move |p: Progress| {
            prog_bar.set_style(ProgressStyle::default_bar().template(STYLE_LOAD));
            prog_bar.set_length(p.total_objects() as u64);

            if p.total_objects() != p.received_objects() {
                prog_bar.set_position(p.received_objects() as u64);
                prog_bar.set_message(&format!(
                    "Downloading object ({})",
                    HumanBytes(p.received_bytes() as u64)
                ));

                if step != Step::Downloading {
                    prog_bar.set_draw_delta(500);
                    prog_bar.reset_eta();
                    step = Step::Downloading;
                }
            } else {
                prog_bar.set_position(p.indexed_objects() as u64);

                if step != Step::Unpacking {
                    prog_bar.reset_eta();
                    prog_bar.set_message("Indexing object");
                    step = Step::Unpacking;
                }
            }
            true
        }
    }

    pub fn start(&self) {
        self.0.reset_elapsed();
        self.report_message("Process is about to begin");
    }

    pub fn report_message(&self, msg: &str) {
        self.0.set_message(msg);
    }
}
