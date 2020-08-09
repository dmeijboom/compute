use std::fs;
use std::env;
use std::process::exit;

use clap::derive::Clap;

mod opts;
mod result;
mod config;
mod ioutils;
mod actions;
mod modules;
mod provisioner;

use config::Config;
use opts::{Opts, Cmd};
use provisioner::Provisioner;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let opts = Opts::parse();

    match opts.cmd {
        Cmd::Apply(opts) => {
            if ioutils::getuid() != 0 {
                eprintln!("compute apply requires root privileges");
                exit(1);
            }

            if let Some(uid) = opts.uid {
                env::set_var("SUDO_UID", uid.to_string());
            } else if env::var("SUDO_UID").is_err() {
                eprintln!("either run compute apply with sudo or set the uid manually");
                exit(1);
            }

            if let Some(gid) = opts.gid {
                env::set_var("SUDO_GID", gid.to_string());
            } else if env::var("SUDO_GID").is_err() {
                eprintln!("either run compute apply with sudo or set the gid manually");
                exit(1);
            }

            println!(">> reading config");

            let contents = fs::read_to_string(&opts.filename)
                .expect("failed to read config");
            let config: Config = json5::from_str(&contents)
                .expect("failed to parse config");
            let mut path = opts.filename;
            path.pop();

            let provisioner = Provisioner::new(opts.skip_downloads);

            println!(">> provisioning");

            match provisioner.run(&path, &config).await {
                Ok(_) => {
                    log::debug!("task finished succesfully");
                    println!(">> finished");
                },
                Err(e) => {
                    log::debug!("task errored: {}", e);
                    println!("provisioning failed:\n  {}", e);
                },
            }
        },
    }
}
