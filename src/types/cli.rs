use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long, short_alias = 'k')]
    pub key_location: Option<PathBuf>
}