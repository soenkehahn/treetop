use crate::process::ProcessWatcher;
use crate::regex::Regex;
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
    #[arg(help = "search pattern for filtering the process tree")]
    pattern: Option<String>,
}

fn main() -> R<()> {
    let args = Args::parse();
    TreetopApp::run(TreetopApp::new(
        ProcessWatcher::new(System::new()),
        args.pattern
            .map(|pattern| ::regex::Regex::new(&pattern).map(crate::Regex::new))
            .transpose()?,
    )?)
}
