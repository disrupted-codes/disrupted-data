use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long)]
    pub key: PathBuf,

    #[arg(long)]
    pub node_ip: Option<String>,

}