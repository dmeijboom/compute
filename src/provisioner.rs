use std::io;
use std::path::Path;
use std::fmt::Display;
use std::marker::Unpin;
use std::future::Future;
use std::process::{Stdio};

use super::config::Config;

use crc32fast::Hasher;
use tokio::fs::{self, File};
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

pub struct Provisioner {
    config: Config,
}

impl Provisioner {
    pub fn new(config: Config) -> Self {
        Self {
            config: config,
        }
    }

    fn checksum(&self, buf: &[u8]) -> u32 {
        let mut hasher = Hasher::new();

        hasher.update(buf);
        hasher.finalize()
    }

    async fn write_file<P, S>(&self, filename: P, source: S) -> io::Result<()>
    where P: AsRef<Path>, S: AsRef<[u8]> + Unpin {
        let right_checksum = self.checksum(source.as_ref());

        let left_source = fs::read_to_string(filename.as_ref()).await?;
        let left_checksum = self.checksum(left_source.as_bytes());

        if right_checksum == left_checksum {
            println!("no changes found for: {}", filename.as_ref().to_string_lossy());
            return Ok(());
        }

        println!("updating file: {}", filename.as_ref().to_string_lossy());
        fs::write(filename, source).await?;

        Ok(())
    }

    async fn install_apt_packages(&self, names: &Vec<String>) -> io::Result<()> {
        let status_file = File::open("/var/lib/dpkg/status").await?;
        let reader = BufReader::new(status_file);
        let mut lines = reader.lines();

        let mut to_be_installed = names.clone();
        let mut package_name = String::new();

        while let Some(line) = lines.next_line().await? {
            if to_be_installed.len() == 0 {
                break;
            }

            let mut fragments = line.split(": ");
            let first_fragment = fragments.next().unwrap();

            match first_fragment {
                "Package" => package_name = fragments.next().unwrap().to_string(),
                "Status" => {
                    if names.contains(&package_name) &&
                        fragments.next() == Some("install ok installed") {
                        to_be_installed.retain(|pkg| pkg != &package_name);
                    }
                },
                _ => continue,
            }
        }

        if to_be_installed.len() == 0 {
            println!("no changes found for apt packages");
            return Ok(());
        }

        println!("installing packages: {}", to_be_installed.join(", "));
        let status = Command::new("apt-get")
            .arg("install")
            .arg("-y")
            .args(&to_be_installed)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .await?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("failed to run apt-get, status code: {}", status.code().unwrap()),
            ));
        }

        Ok(())
    }

    async fn wrap<F, E>(&self, func: F)
    where F: Future<Output = Result<(), E>>, E: Display {
        match func.await {
            Ok(_) => log::debug!("task finished succesfully"),
            Err(e) => log::error!("task errored: {}", e),
        }
    }

    pub async fn run(&self) {
        println!(">> provisioning");

        let write_file = |filename, source|
            self.wrap(self.write_file(filename, source));

        let hostname_tmpl = format!("{}\n", self.config.networking.hostname);

        tokio::join!(
            write_file("/etc/hostname", hostname_tmpl.as_bytes()),
            self.wrap(self.install_apt_packages(&self.config.apt.packages)),
        );

        println!(">> finished");
    }
}
