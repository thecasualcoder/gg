use colored::Colorize;
use git2::{Error, Progress};
use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};

use crate::git::GitAction;

pub struct ProgressTracker(MultiProgress);

impl ProgressTracker {
    pub fn new() -> Self {
        let bars = MultiProgress::new();
        bars.set_move_cursor(true);
        Self(bars)
    }

    pub fn new_bar(&self, remote_url: &str) -> ProgressReporter {
        let prog_bar = self
            .0
            .add(ProgressBar::new_spinner())
            .with_style(ProgressStyle::default_bar().template(STYLE_PRELOAD));

        prog_bar.set_prefix(&remote_url.blue());
        prog_bar.set_message(&"Waiting for process to begin".cyan());
        prog_bar.set_draw_delta(100);

        ProgressReporter(prog_bar)
    }

    pub fn start_task(&self, mut action: impl GitAction + Send + 'static) {
        let progress_bar = self.new_bar(&action.get_name());
        rayon::spawn(move || action.do_git_action(progress_bar));
    }

    pub fn join(self) -> Result<(), std::io::Error> {
        self.0.join()
    }

    // internalAutoTick ?
}

pub struct ProgressReporter(ProgressBar);

const STYLE_PRELOAD: &str = "{prefix:>40!.blue} {spinner} {wide_msg:.cyan}";
const STYLE_LOAD: &str = "{prefix:>40!.blue} {msg} {wide_bar} {percent:>3}% {eta}";
const STYLE_DONE: &str = "{prefix:>40!.blue} {wide_msg} {elapsed_precise}";
const STYLE_ERROR: &str = "{prefix:>40!.blue} {wide_msg:.red}";

impl ProgressReporter {
    pub fn finalize(self, status: &str) {
        self.0
            .set_style(ProgressStyle::default_bar().template(STYLE_DONE));
        self.0.finish_with_message(status);
    }

    pub fn abandon(self, err: Error) {
        self.0
            .set_style(ProgressStyle::default_bar().template(STYLE_ERROR));
        // self.0.println(format!("{}", err));
        self.0.abandon_with_message(err.message());
    }

    pub fn get_callback(&self) -> Box<dyn FnMut(Progress) -> bool> {
        let prog_bar = self.0.clone();
        Box::new(move |p: Progress| {
            prog_bar.set_style(ProgressStyle::default_bar().template(STYLE_LOAD));
            prog_bar.set_length(p.total_objects() as u64);

            if p.total_objects() != p.received_objects() {
                prog_bar.set_position(p.received_objects() as u64);
                prog_bar.set_message(&format!(
                    "Downloading object ({})",
                    HumanBytes(p.received_bytes() as u64)
                ));
            } else {
                // Todo reset ETA
                prog_bar.set_position(p.indexed_objects() as u64);
                prog_bar.set_message("Indexing object");
            }
            true
        })
    }
}
