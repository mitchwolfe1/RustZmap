mod cli;
mod scanner;

use clap::Parser;
use cli::Opt;
use scanner::scan_subnet;
use std::time::Duration;
use anyhow::Result;

fn main() -> anyhow::Result<()> {
    // let opt = Opt::parse();

    scan_subnet("10.0.0.0/24", 1945, Duration::from_millis(100))
}


