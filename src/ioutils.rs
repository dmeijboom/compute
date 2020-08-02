use std::path::Path;
use std::ffi::CString;
use std::fs::Permissions;
use std::io::{Result, Error, ErrorKind};
use std::os::unix::prelude::PermissionsExt;

use tokio::fs;
use tokio::task::spawn_blocking;

pub fn getuid() -> u32 {
    unsafe {
        libc::getuid()
    }
}

pub async fn chmod<P>(filename: P, mode: u32) -> Result<()>
where P: AsRef<Path> {
    fs::set_permissions(
        filename.as_ref(),
        Permissions::from_mode(mode),
    ).await
}

pub async fn chown<S>(filename: S, uid: u32, gid: u32) -> Result<()>
where S: AsRef<str> + Send {
    let chown_filename = CString::new(filename.as_ref().to_string())
        .unwrap();
    let result = spawn_blocking(move || {
        unsafe {
            libc::chown(chown_filename.as_ptr() as *const i8, uid, gid)
        }
    }).await?;

    if result > 0 {
        return Err(Error::new(
            ErrorKind::Other,
            format!("failed to change ownership of file {}: code {}", filename.as_ref(), result),
        ));
    }

    Ok(())
}
