use std::io::{Result, Error, ErrorKind};

use async_stream::stream;
use tokio::stream::Stream;
use tokio::fs::{self, File};
use tokio::stream::StreamExt;
use tokio::io::{BufReader, AsyncBufReadExt};

use super::{write_file, run_cmd};
use crate::config::apt::AptRepository;

pub async fn update_packages() -> Result<()> {
    run_cmd("apt-get", &["update"], true, None).await
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

pub fn list_installed_packages() -> impl Stream<Item = Result<String>> {
    stream! {
        let status_file = File::open("/var/lib/dpkg/status").await?;
        let reader = BufReader::new(status_file);
        let mut lines = reader.lines();
        let mut package_name = String::new();

        while let Some(line) = lines.next_line().await? {
            let mut fragments = line.split(": ");
            let first_fragment = fragments.next().unwrap();

            match first_fragment {
                "Package" => package_name = fragments.next().unwrap().to_string(),
                "Status" => {
                    if fragments.next() == Some("install ok installed") {
                        yield Ok(package_name.clone());
                    }
                },
                _ => continue,
            }
        }
    }
}

pub async fn install_packages(names: &Vec<String>) -> Result<()> {
    let mut to_be_installed = names.clone();
    let packages = list_installed_packages();
    tokio::pin!(packages);

    while let Some(Ok(package)) = packages.next().await {
        if to_be_installed.len() == 0 {
            break;
        }

        to_be_installed.retain(|pkg| pkg != &package);
    }

    if to_be_installed.len() == 0 {
        println!("no changes found for apt packages");
        return Ok(());
    }

    println!("updating packages");
    update_packages().await?;

    println!("installing packages: {}", to_be_installed.join(", "));
    run_cmd("apt-get", &[&["install", "--no-install-recommends", "-y"][..], &to_be_installed
        .iter()
        .map(|pkg| pkg.as_str())
        .collect::<Vec<_>>()[..],
    ].concat(), true, None).await?;

    Ok(())
}
