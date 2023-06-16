use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;

pub struct Bar {
    progress_bar: ProgressBar,
}

impl Bar {
    pub fn new(n: i64) -> Self {
        let progress_bar = ProgressBar::new(n as u64);

        progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

        Self { progress_bar }
    }

    pub fn set_position(&self, n: u64) {
        self.progress_bar.set_position(n)
    }

    pub fn finish_with_message(&self, msg: &'static str) {
        self.progress_bar.finish_with_message(msg)
    }
    pub fn set_message(&self, msg: String) {
        self.progress_bar.set_message(msg)
    }

    pub fn print(&self, msg: String) {
        self.progress_bar.println(msg)
    }
}
