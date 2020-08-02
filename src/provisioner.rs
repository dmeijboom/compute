use std::str;
use std::path::PathBuf;

use super::actions;
use super::ioutils::chown;
use super::templates::Template;
use super::config::{Config, files::TemplateSource};

use tera::Context;

type Error = Box<dyn std::error::Error>;

pub struct Provisioner {
    config: Config,
    root_dir: PathBuf,
    skip_downloads: bool,
}

impl Provisioner {
    pub fn new(root_dir: PathBuf, skip_downloads: bool, config: Config) -> Self {
        Self {
            config: config,
            root_dir: root_dir,
            skip_downloads: skip_downloads,
        }
    }

    async fn configure_networking(&self) -> Result<(), Error> {
        if let Some(hostname) = &self.config.networking.hostname {
            let mut hostname_ctx = Context::new();
            hostname_ctx.insert("hostname", hostname);

            actions::write_template(
                "/etc/hostname",
                Template::NetworkingHostname,
                hostname_ctx,
            ).await?;
        }

        if self.config.networking.hosts.len() > 0 {
            let mut hosts_ctx = Context::new();
            hosts_ctx.insert("hostname", &self.config.networking.hostname);
            hosts_ctx.insert("hosts", &self.config.networking.hosts);

            actions::write_template(
                "/etc/hosts",
                Template::NetworkingHosts,
                hosts_ctx,
            ).await?;
        }

        Ok(())
    }

    async fn configure_apt(&self) -> Result<(), Error> {
        for repo in &self.config.apt.repositories {
            actions::add_repository(repo).await?;
        }

        actions::install_packages(&self.config.apt.packages).await?;

        Ok(())
    }

    async fn configure_app_image(&self) -> Result<(), Error> {
        for app in &self.config.app_image.apps {
            actions::install_app_image_app(app).await?;
        }

        Ok(())
    }

    async fn configure_scripts(&self) -> Result<(), Error> {
        for script in &self.config.scripts {
            actions::run_script(&script).await?;
        }

        Ok(())
    }

    async fn configure_files(self) -> Result<(), Error> {
        for file in self.config.files.into_iter() {
            let contents = match file.source() {
                TemplateSource::Local(template) => {
                    let mut src = self.root_dir.clone();
                    src.push(&template);

                    tokio::fs::read(src).await?
                },
                TemplateSource::S3(file) => {
                    if self.skip_downloads {
                        println!("skipping download for: s3/{}/{}", file.bucket_name, file.path);
                        continue;
                    }

                    actions::download_file(
                        file.path.clone(),
                        self.config.s3.buckets
                            .iter()
                            .filter(|b| b.name == file.bucket_name)
                            .next()
                            .unwrap(),
                    ).await?
                },
            };

            if actions::write_user_template(
                &file.path,
                str::from_utf8(&contents)?,
                file.context,
            ).await? {
                if let Some((uid, gid)) = file.owner {
                    log::info!("changing ownership of: {} to {}:{}", file.path, uid, gid);
                    chown(&file.path, uid, gid).await?;
                }
            }
        }

        Ok(())
    }

    async fn configure_all(self) -> Result<(), Error> {
        self.configure_networking().await?;
        self.configure_apt().await?;
        self.configure_app_image().await?;
        self.configure_scripts().await?;
        self.configure_files().await?;

        Ok(())
    }

    pub async fn run(self) {
        println!(">> provisioning");

        match self.configure_all().await {
            Ok(_) => log::debug!("task finished succesfully"),
            Err(e) => log::error!("task errored: {}", e),
        }

        println!(">> finished");
    }
}
