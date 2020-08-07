use std::str;
use std::path::{PathBuf, Path};

use tera::{Map, Value};
use async_recursion::async_recursion;

use super::actions;
use super::modules::load_module;
use super::ioutils::{chmod, chown};
use super::config::{Config, files::TemplateSource};

type Error = Box<dyn std::error::Error>;

pub struct Provisioner {
    skip_downloads: bool,
}

impl Provisioner {
    pub fn new(skip_downloads: bool) -> Self {
        Self {
            skip_downloads: skip_downloads,
        }
    }

    async fn configure_apt(&self, _root_dir: &Path, config: &Config) -> Result<(), Error> {
        for repo in &config.apt.repositories {
            actions::add_repository(repo).await?;
        }

        actions::install_packages(&config.apt.packages).await?;

        Ok(())
    }

    async fn configure_app_image(&self, _root_dir: &Path, config: &Config) -> Result<(), Error> {
        for app in &config.app_image.apps {
            actions::install_app_image_app(app).await?;
        }

        Ok(())
    }

    async fn configure_scripts(&self, _root_dir: &Path, config: &Config) -> Result<(), Error> {
        for script in &config.scripts {
            actions::run_script(script).await?;
        }

        Ok(())
    }

    async fn configure_files(&self, root_dir: &Path, config: &Config, vars: Option<Map<String, Value>>) -> Result<(), Error> {
        for file in &config.files {
            let updated = match file.source() {
                TemplateSource::Local(template) => {
                    let mut src = PathBuf::from(root_dir);
                    src.push(&template);

                    log::info!("reading local template file: {:?}", src);

                    let contents = tokio::fs::read(src).await?;

                    actions::write_template(
                        &file.path,
                        str::from_utf8(&contents)?,
                        match &vars {
                            None => file.context.clone(),
                            Some(vars) => {
                                let mut ctx = file.context.clone();

                                ctx.insert("vars".to_string(), Value::Object(vars.clone()));
                                ctx
                            },
                        },
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
    pub async fn configure_modules(&self, _root_dir: &Path, config: &Config) -> Result<(), Error> {
        for (name, config) in &config.modules {
            log::info!("loading module: {}", name);

            let (module_root, module) = load_module(name).await?;

            if let Value::Object(vars) = config {
                self.configure_all(
                    &module_root,
                    &module.config,
                    Some(vars.clone()),
                ).await?;
            } else {
                println!("skipping module because of invalid type for vars: {}", name);
            }
        }

        Ok(())
    }

    #[async_recursion]
    pub async fn configure_all(&self, root_dir: &Path, config: &Config, vars: Option<Map<String, Value>>) -> Result<(), Error> {
        self.configure_modules(root_dir, config).await?;
        self.configure_apt(root_dir, config).await?;
        self.configure_app_image(root_dir, config).await?;
        self.configure_scripts(root_dir, config).await?;
        self.configure_files(root_dir, config, vars).await?;

        Ok(())
    }

    pub async fn run(&self, root_dir: PathBuf, config: &Config) {
        println!(">> provisioning");

        match self.configure_all(&root_dir, config, None).await {
            Ok(_) => log::debug!("task finished succesfully"),
            Err(e) => log::error!("task errored: {}", e),
        }

        println!(">> finished");
    }
}
