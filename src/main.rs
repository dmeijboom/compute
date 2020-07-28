use std::fs;
use std::env;
use std::process::exit;

use clap::derive::Clap;

mod opts;
mod config;
mod actions;
mod templates;
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
            unsafe {
                if libc::getuid() != 0 {
                    eprintln!("compute apply requires root privileges");
                    exit(1);
                }
            }

            if let Some(uid) = opts.uid {
                env::set_var("SUDO_UID", uid.to_string());
            } else if env::var("SUDO_UID").is_err() {
                eprintln!("either run compute apply with sudo or set the uid manually");
                exit(1);
            }

            println!(">> reading config");

            let contents = fs::read_to_string(opts.filename)
                .expect("failed to read config");
            let config: Config = json5::from_str(&contents)
                .expect("failed to parse config");
            let provisioner = Provisioner::new(config);

            provisioner.run().await;
        },
    }
}
