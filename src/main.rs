use std::fs;

use clap::derive::Clap;

mod opts;
mod config;
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
