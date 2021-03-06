use std::str;
use std::path::{PathBuf, Path};

use async_recursion::async_recursion;

use super::actions;
use super::result::Result;
use super::modules::load_module;
use super::ioutils::{chmod, chown};
use super::config::{Config, files::TemplateSource};

pub struct Provisioner {
    skip_downloads: bool,
}

impl Provisioner {
    pub fn new(skip_downloads: bool) -> Self {
        Self {
            skip_downloads: skip_downloads,
        }
    }

    async fn configure_apt(&self, _root_dir: &Path, config: &Config) -> Result<()> {
        for repo in &config.apt.repositories {
            actions::add_repository(repo).await?;
        }

        actions::install_packages(&config.apt.packages).await?;

        Ok(())
    }

    async fn configure_scripts(&self, _root_dir: &Path, config: &Config) -> Result<()> {
        for script in &config.scripts {
            actions::run_script(script).await?;
        }

        Ok(())
    }

    async fn configure_files(&self, root_dir: &Path, config: &Config) -> Result<()> {
        for file in &config.files {
            let updated = match file.source() {
                TemplateSource::Local(template) => {
                    let mut src = PathBuf::from(root_dir);
                    src.push(&template);

                    log::info!("reading local template file: {:?}", src);

                    let contents = tokio::fs::read(&src).await?;

                    actions::write_template(
                        &template,
                        &file.path,
                        str::from_utf8(&contents)?,
                        file.context.clone(),
                    ).await?
                },
                TemplateSource::S3(s3file) => {
                    if self.skip_downloads {
                        println!("skipping download for: s3/{}/{}", s3file.bucket_name, s3file.path);
                        continue;
                    }

                    log::info!("downloading remote file: s3/{}/{}", s3file.bucket_name, s3file.path);

                    let contents = actions::download_file(
                        s3file.path.clone(),
                        config.s3.buckets
                            .iter()
                            .filter(|b| b.name == s3file.bucket_name)
                            .next()
                            .unwrap(),
                    ).await?;

                    actions::write_file(
                        &file.path,
                        &contents,
                    ).await?
                },
            };

            if updated {
                if let Some((uid, gid)) = file.owner {
                    log::info!("changing ownership of: {} to {}:{}", file.path, uid, gid);
                    chown(&file.path, uid, gid).await?;
                }

                if let Some(mode) = file.mode {
                    log::info!("changing mode of: {} to {}", file.path, mode);
                    chmod(&file.path, mode).await?;
                }
            }
        }

        Ok(())
    }

    #[async_recursion]
    pub async fn configure_modules(&self, _root_dir: &Path, config: &Config) -> Result<()> {
        for (name, config) in &config.modules {
            log::info!("loading module: {}", name);
            println!(">>> configuring module {}", name);

            let (module_root, module) = load_module(name, config.clone()).await?;

            self.run(
                &module_root,
                &module.config,
            ).await?;
        }

        Ok(())
    }

    #[async_recursion]
    pub async fn run(&self, root_dir: &Path, config: &Config) -> Result<()> {
        self.configure_modules(root_dir, config).await?;

        println!(">>> configuring apt");
        self.configure_apt(root_dir, config).await?;

        println!(">>> configuring scripts");
        self.configure_scripts(root_dir, config).await?;

        println!(">>> configuring files");
        self.configure_files(root_dir, config).await?;

        Ok(())
    }
}
