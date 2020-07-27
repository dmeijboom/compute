use std::env;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::io::{Result, Error, ErrorKind};

use tokio::fs;
use tokio::stream::StreamExt;
use tokio::task::spawn_blocking;

use crate::config::app_image::App;
use super::{list_installed_packages, run_cmd, CmdOpts};

async fn chown(filename: String, uid: u32, gid: u32) -> Result<()> {
    let original_filename = filename.clone();
    let result = spawn_blocking(move || {
        unsafe {
            return libc::chown(filename.as_ptr() as *const i8, uid, gid);
        }
    }).await?;

    if result != 0 || result != -1 {
        return Err(Error::new(
            ErrorKind::Other,
            format!("failed to change ownership of file {}: code {}", original_filename, result),
        ));
    }

    Ok(())
}

pub async fn install_app_image_app(app: &App) -> Result<()> {
    let unprivileged_uid = env::var("SUDO_UID")
        .unwrap()
        .parse()
        .unwrap();
    let unprivileged_gid = env::var("SUDO_GID")
        .unwrap()
        .parse()
        .unwrap();

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

    let temp_dir = format!("{}/compute/builds/{}", env::temp_dir().as_path().display(), app.name);
    let temp_path = &Path::new(&temp_dir);
    let app_image_filename = format!("{}/{}.AppImage", temp_dir, app.name);

    fs::create_dir_all(&temp_dir).await?;
    chown(temp_dir.clone(), unprivileged_uid, unprivileged_gid).await?;

    log::info!("downloading AppImage file: {}", app.url);
    let body = reqwest::get(&app.url)
        .await.map_err(|e| Error::new(ErrorKind::Other, e))?
        .bytes()
        .await.map_err(|e| Error::new(ErrorKind::Other, e))?;
    fs::write(&app_image_filename, body).await?;
    chown(app_image_filename.clone(), unprivileged_uid, unprivileged_gid).await?;

    log::info!("changing executable flag for AppImage file: {}", app_image_filename);
    let mut perms = fs::metadata(&app_image_filename).await?.permissions();
    perms.set_mode(0o775);
    fs::set_permissions(&app_image_filename, perms).await?;

    log::info!("extracting squashfs filesystem to: {}/squashfs-root", temp_dir);
    run_cmd(CmdOpts {
        name: "sh",
        args: &["-c", format!("./{}.AppImage --appimage-extract", app.name).as_str()],
        cwd: Some(temp_path),
        privileged: false,
        ..CmdOpts::default()
    }).await?;

    let package_name = format!("{}-{}.0-1", app.name, app.version);
    log::info!("creating debian package: {}", package_name);
    let package_dir = format!("{}/{}", temp_dir, package_name);

    fs::rename(format!("{}/squashfs-root", temp_dir), &package_dir).await?;

    let debian_dir = format!("{}/DEBIAN", package_dir);
    fs::create_dir_all(&debian_dir).await?;
    chown(debian_dir.clone(), unprivileged_uid, unprivileged_gid).await?;

    let debian_control_file = format!("{}/DEBIAN/control", package_dir);
    fs::write(&debian_control_file, format!(
        "Package: app-image-{}\nVersion: {}.0-1\nSection: base\nPriority: optional\nArchitecture: amd64\nMaintainer: compute\nDescription:\n  Auto-generated package from an AppImage binary\n",
        app.name,
        app.version,
    )).await?;
    chown(debian_control_file.clone(), unprivileged_uid, unprivileged_gid).await?;

    // remove all root files as they are probably AppImage specific
    let mut read_dir = fs::read_dir(&package_dir).await?;

    while let Some(Ok(entry)) = read_dir.next().await {
        let file_type = entry.file_type().await?;

        if file_type.is_file() {
            fs::remove_file(entry.path()).await?;
        }
    }

    log::info!("building debian package: {}.deb", package_name);
    run_cmd(CmdOpts {
        name: "dpkg-deb",
        args: &["--build", package_name.as_str()],
        cwd: Some(&temp_path),
        ..CmdOpts::default()
    }).await?;

    log::info!("installing debian package: {}.deb", package_name);
    run_cmd(CmdOpts {
        name: "dpkg",
        args: &["-i", format!("{}.deb", package_name).as_str()],
        inherit_output: true,
        cwd: Some(&temp_path),
        ..CmdOpts::default()
    }).await?;

    Ok(())
}
