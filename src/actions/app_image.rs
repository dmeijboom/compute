use std::env::temp_dir;
use std::os::unix::fs::PermissionsExt;
use std::io::{Result, Error, ErrorKind};

use tokio::fs;
use tokio::stream::StreamExt;

use crate::config::app_image::App;
use super::{list_installed_packages, run_cmd};

pub async fn install_app_image_app(app: &App) -> Result<()> {
    let package_name = format!("app-image-{}", app.name);
    let packages = list_installed_packages();
    tokio::pin!(packages);

    while let Some(Ok(package)) = packages.next().await {
        if package == package_name {
            println!("AppImage {} already installed", app.name);
            return Ok(());
        }
    }

    println!("installing AppImage {}", app.name);

    let temp_dir = format!("{}/compute/builds/{}", temp_dir().as_path().display(), app.name);
    let app_image_filename = format!("{}/{}.AppImage", temp_dir, app.name);

    fs::create_dir_all(&temp_dir).await?;

    log::info!("downloading AppImage file: {}", app.url);
    let body = reqwest::get(&app.url)
        .await.map_err(|e| Error::new(ErrorKind::Other, e))?
        .bytes()
        .await.map_err(|e| Error::new(ErrorKind::Other, e))?;
    fs::write(&app_image_filename, body).await?;

    log::info!("changing executable flag for AppImage file: {}", app_image_filename);
    let mut perms = fs::metadata(&app_image_filename).await?.permissions();
    perms.set_mode(0o775);
    fs::set_permissions(&app_image_filename, perms).await?;

    log::info!("extracting squashfs filesystem to: {}/squashfs-root", temp_dir);
    run_cmd(
        "sh",
        &["-c", format!("./{}.AppImage --appimage-extract", app.name).as_str()],
        false,
        Some(temp_dir.clone()),
    ).await?;

    let package_name = format!("{}-{}.0-1", app.name, app.version);
    log::info!("creating debian package: {}", package_name);
    let package_dir = format!("{}/{}", temp_dir, package_name);

    fs::rename(format!("{}/squashfs-root", temp_dir), &package_dir).await?;
    fs::create_dir_all(format!("{}/DEBIAN", package_dir)).await?;
    fs::write(format!("{}/DEBIAN/control", package_dir), format!(
        "Package: app-image-{}\nVersion: {}.0-1\nSection: base\nPriority: optional\nArchitecture: amd64\nMaintainer: compute\n",
        app.name,
        app.version,
    )).await?;

    // remove all root files as they are probably AppImage specific
    let mut read_dir = fs::read_dir(&package_dir).await?;

    while let Some(Ok(entry)) = read_dir.next().await {
        let file_type = entry.file_type().await?;

        if file_type.is_file() {
            fs::remove_file(entry.path()).await?;
        }
    }

    log::info!("building debian package: {}.deb", package_name);
    run_cmd(
        "dpkg-deb",
        &["--build", package_name.as_str()],
        false,
        Some(temp_dir.clone()),
    ).await?;

    log::info!("installing debian package: {}.deb", package_name);
    run_cmd(
        "dpkg",
        &["-i", format!("{}.deb", package_name).as_str()],
        true,
        Some(temp_dir.clone()),
    ).await?;

    Ok(())
}
