use super::actions;
use super::config::Config;

type Error = Box<dyn std::error::Error>;

pub struct Provisioner {
    config: Config,
}

impl Provisioner {
    pub fn new(config: Config) -> Self {
        Self {
            config: config,
        }
    }

    async fn configure_networking(&self) -> Result<(), Error> {
        let hostname_tmpl = format!("{}\n", self.config.networking.hostname);

        actions::write_file("/etc/hostname", hostname_tmpl.as_bytes()).await?;

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

    async fn configure_all(&self) -> Result<(), Error> {
        self.configure_networking().await?;
        self.configure_apt().await?;
        self.configure_app_image().await?;

        Ok(())
    }

    pub async fn run(&self) {
        println!(">> provisioning");

        match self.configure_all().await {
            Ok(_) => log::debug!("task finished succesfully"),
            Err(e) => log::error!("task errored: {}", e),
        }

        println!(">> finished");
    }
}
