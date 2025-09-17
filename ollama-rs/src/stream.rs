use futures_util::{stream::BoxStream, StreamExt};
use reqwest::Response;

use crate::error::OllamaError;

mod bytes_lines_stream;

pub async fn map_response<T: serde::de::DeserializeOwned>(
    res: Response,
) -> crate::error::Result<BoxStream<'static, crate::error::Result<T>>> {
    if res.status().is_success() {
        Ok(Box::pin(bytes_lines_stream::lines(res.bytes_stream()).map(
            |res| {
                let bytes = res?;
                serde_json::from_slice::<T>(&bytes).map_err(|e| {
                    match serde_json::from_slice::<crate::error::InternalOllamaError>(&bytes) {
                        Ok(err) => OllamaError::InternalError(err),
                        Err(_) => e.into(),
                    }
                })
            },
        )))
    } else {
        Err(OllamaError::Other(res.text().await?))
    }
}
