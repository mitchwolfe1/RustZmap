mod cli;
mod scanner;

use clap::Parser;
use cli::Opt;
use scanner::scan_subnet;

fn main() {
    let opt = Opt::parse();

    scan_subnet(&opt.subnet, opt.port).unwrap();
}


