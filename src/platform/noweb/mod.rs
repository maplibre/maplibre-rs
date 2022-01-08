//! Module which is used target platform is not web related.

use crate::error::Error;

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err.to_string())
    }
}

pub async fn download(url: String) -> Result<Vec<u8>, Error> {
    let body = reqwest::get(url).await?.bytes().await?;
    Ok(Vec::from(body.as_ref()))
}
