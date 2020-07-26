use std::io;
use std::path::Path;
use std::marker::Unpin;

use tokio::fs;
use crc32fast::Hasher;

fn checksum(buf: &[u8]) -> u32 {
    let mut hasher = Hasher::new();

    hasher.update(buf);
    hasher.finalize()
}

pub async fn write_file<P, S>(filename: P, source: S) -> io::Result<bool>
where P: AsRef<Path>, S: AsRef<[u8]> + Unpin {
    let right_checksum = checksum(source.as_ref());

    if filename.as_ref().exists() {
        let left_source = fs::read_to_string(filename.as_ref()).await?;
        let left_checksum = checksum(left_source.as_bytes());

        if right_checksum == left_checksum {
            println!("no changes found for: {}", filename.as_ref().to_string_lossy());
            return Ok(false);
        }
    }

    println!("updating file: {}", filename.as_ref().to_string_lossy());
    fs::write(filename, source).await?;

    Ok(true)
}
