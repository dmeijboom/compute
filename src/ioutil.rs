use std::io::{Result, Error, ErrorKind};

use tokio::task::spawn_blocking;

pub fn getuid() -> u32 {
    unsafe {
        libc::getuid()
    }
}

pub async fn chown<S>(filename: S, uid: u32, gid: u32) -> Result<()>
where S: AsRef<str> + Send {
    let chown_filename = filename.as_ref().to_string();
    let result = spawn_blocking(move || {
        unsafe {
            return libc::chown(chown_filename.as_ptr() as *const i8, uid, gid);
        }
    }).await?;

    if result != 0 && result != -1 {
        return Err(Error::new(
            ErrorKind::Other,
            format!("failed to change ownership of file {}: code {}", filename.as_ref(), result),
        ));
    }

    Ok(())
}
