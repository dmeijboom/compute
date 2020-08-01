use std::path::Path;
use std::marker::Unpin;
use std::io::{Result, Error, ErrorKind};

use crc32fast::Hasher;
use tokio::io::AsyncWriteExt;
use tokio::fs::{self, OpenOptions};
use tera::{Tera, Context, Map, Value};

use crate::templates::Template;

fn checksum(buf: &[u8]) -> u32 {
    let mut hasher = Hasher::new();

    hasher.update(buf);
    hasher.finalize()
}

pub async fn write_file<P, S>(filename: P, source: S) -> Result<bool>
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
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filename)
        .await?;

    let mut perms = file
        .metadata()
        .await?
        .permissions();
    perms.set_readonly(true);

    file.write_all(source.as_ref()).await?;
    file.set_permissions(perms).await?;

    Ok(true)
}

pub async fn write_template<P>(filename: P, template: Template, ctx: Context) -> Result<bool>
where P: AsRef<Path> {
    let contents = template.get_contents();
    let contents = Tera::one_off(contents, &ctx, false)
        .map_err(|e| Error::new(
            ErrorKind::Other,
            format!("failed to render template: {}", e),
        ))?;

    write_file(filename, contents.as_bytes()).await
}

pub async fn write_user_template<P, S>(filename: P, source: S, ctx: Map<String, Value>) -> Result<bool>
where P: AsRef<Path>, S: AsRef<str> {
    let contents = Tera::one_off(
        source.as_ref(),
        &Context::from_value(Value::Object(ctx)).map_err(|e| Error::new(
            ErrorKind::Other,
            format!("failed to get context from map: {}", e),
        ))?,
        false,
    )
        .map_err(|e| Error::new(
            ErrorKind::Other,
            format!("failed to render template: {}", e),
        ))?;

    write_file(filename, contents.as_bytes()).await
}
