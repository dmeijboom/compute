use std::env;

use tokio::io::AsyncReadExt;
use rusoto_core::Region;
use rusoto_s3::{S3, S3Client, GetObjectRequest};

use crate::result::Result;
use crate::config::s3::Bucket;

pub async fn download_file(path: String, bucket: &Bucket) -> Result<Vec<u8>> {
    log::info!("downloading: {:?} from s3/{}", path, bucket.name);

    let client = S3Client::new(Region::Custom {
        name: "eu-central-1".to_string(),
        endpoint: bucket.endpoint.clone(),
    });

    env::set_var("AWS_ACCESS_KEY_ID", bucket.key.as_str());
    env::set_var("AWS_SECRET_ACCESS_KEY", bucket.secret.as_str());

    let object = client.get_object(GetObjectRequest {
        bucket: bucket.name.clone(),
        key: path,
        ..GetObjectRequest::default()
    })
        .await?;

    let mut reader = object.body.unwrap().into_async_read();
    let mut contents = vec![];

    reader.read_to_end(&mut contents).await?;

    Ok(contents)
}
