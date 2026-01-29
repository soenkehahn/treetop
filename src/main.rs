use crate::process::ProcessWatcher;
use crate::treetop_app::TreetopApp;
use clap::Parser;
use std::error::Error;
use sysinfo::System;

mod process;
mod regex;
mod tree;
mod treetop_app;
mod tui_app;
mod utils;

type R<A> = Result<A, Box<dyn Error>>;

#[derive(Parser, Debug)]
struct Args {
    /// Search pattern for filtering the process tree
    pattern: Option<String>,

    #[arg(long)]
    /// Prevents treetop from hiding itself
    ///
    /// By default treetop will hide itself (i.e. its own process) if and only if matched on
    /// process arguments. Otherwise treetop would often show itself when passing a search pattern
    /// as an argument. This is usually not useful. This flag makes sure treetop always shows
    /// itself when matched.
    dont_hide_self: bool,
}

fn main() -> R<()> {
    let args = Args::parse();
    TreetopApp::run(TreetopApp::new(ProcessWatcher::new(System::new()), args)?)
}
