use clap::Parser;

#[derive(Parser)]
pub struct Opt {
    pub subnet: String,
    pub port: u16,
}

