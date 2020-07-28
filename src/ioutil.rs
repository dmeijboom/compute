use std::io::{Result, Error, ErrorKind};

use tokio::task::spawn_blocking;

pub fn getuid() -> u32 {
    unsafe {
        libc::getuid()
    }
}

pub async fn chown(filename: String, uid: u32, gid: u32) -> Result<()> {
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
