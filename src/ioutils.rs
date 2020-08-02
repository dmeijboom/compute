use std::ffi::CString;
use std::io::{Result, Error, ErrorKind};

use tokio::task::spawn_blocking;

pub fn getuid() -> u32 {
    unsafe {
        libc::getuid()
    }
}

pub async fn chown<S>(filename: S, uid: u32, gid: u32) -> Result<()>
where S: AsRef<str> + Send {
    let chown_filename = CString::new(filename.as_ref().to_string())
        .unwrap();
    let result = spawn_blocking(move || {
        unsafe {
            let result = libc::chown(chown_filename.as_ptr() as *const i8, uid, gid);

            if result > 0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to change ownership of file {}: code {}", filename.as_ref(), result),
                ));
            }
        }
    }).await?;

    Ok(())
}
