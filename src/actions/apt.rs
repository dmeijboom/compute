use std::process::{Stdio};
use std::io::{Result, Error, ErrorKind};

use tokio::fs::{self, File};
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

use super::write_file;
use crate::config::apt::AptRepository;

async fn run_apt_get(args: &[&str]) -> Result<()> {
    log::info!("running: apt-get {}", args.join(" "));

    let status = Command::new("apt-get")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .await?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("failed to run apt-get, status code: {}", status.code().unwrap()),
        ));
    }

    Ok(())
}

pub async fn update_packages() -> Result<()> {
    run_apt_get(&["update"]).await
}

pub async fn add_repository(repo: &AptRepository) -> Result<()> {
    let tmpl = format!(
        "deb [signed-by=/usr/share/keyrings/{}.gpg] {} {} {}\n",
        repo.name,
        repo.url,
        repo.distro,
        repo.components.join(" "),
    );

    if write_file(
        format!("/etc/apt/sources.list.d/{}.list", repo.name),
        tmpl.as_bytes(),
    ).await? {
        let body = reqwest::get(&repo.key_url)
            .await.map_err(|e| Error::new(ErrorKind::Other, e))?
            .bytes()
            .await.map_err(|e| Error::new(ErrorKind::Other, e))?;

        fs::write(format!("/usr/share/keyrings/{}.gpg", repo.name), body).await?;
    }

    println!("no changes found for apt repository: {}", repo.name);
    Ok(())
}

pub async fn install_packages(names: &Vec<String>) -> Result<()> {
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

    println!("updating packages");
    update_packages().await?;

    println!("installing packages: {}", to_be_installed.join(", "));
    run_apt_get(&[&["install", "--no-install-recommends", "-y"][..], &to_be_installed
        .iter()
        .map(|pkg| pkg.as_str())
        .collect::<Vec<_>>()[..],
    ].concat()).await?;

    Ok(())
}
