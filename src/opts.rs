use std::path::PathBuf;

use clap::Clap;

#[derive(Debug, Clap)]
#[clap(version = "0.1", author = "Dillen <info@dillen.dev>")]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Clap)]
pub struct ApplyOpts {
    #[clap(short)]
    pub filename: PathBuf,
    #[clap(short)]
    pub uid: Option<u32>,
    #[clap(short)]
    pub gid: Option<u32>,
    #[clap(long)]
    pub skip_downloads: bool,
}

#[derive(Debug, Clap)]
pub enum Cmd {
    #[clap()]
    Apply(ApplyOpts),
}
