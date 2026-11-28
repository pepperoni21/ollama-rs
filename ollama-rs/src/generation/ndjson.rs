use async_stream::stream;
use serde::de::DeserializeOwned;
use tokio_stream::{Stream, StreamExt};

use crate::error::{InternalOllamaError, OllamaError, Result};

pub(super) fn records<T: DeserializeOwned + Send>(
    res: reqwest::Response,
) -> impl Stream<Item = Result<T>> + Send {
    stream! {
        let mut line = Vec::new();
        let mut stream = res.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(err) => {
                    yield Err(err.into());
                    return;
                }
            };
            // HTTP chunks can split a UTF-8 code point as well as a JSON record.
            for part in chunk.split_inclusive(|&byte| byte == b'\n') {
                line.extend_from_slice(part);
                if part.ends_with(b"\n") {
                    if !line.iter().all(u8::is_ascii_whitespace) {
                        match parse_record(&line) {
                            Ok(response) => yield Ok(response),
                            Err(err) => {
                                yield Err(err);
                                return;
                            }
                        }
                    }
                    line.clear();
                }
            }
        }
        if !line.iter().all(u8::is_ascii_whitespace) {
            yield parse_record(&line);
        }
    }
}

fn parse_record<T: DeserializeOwned>(line: &[u8]) -> Result<T> {
    match serde_json::from_slice::<T>(line) {
        Ok(response) => Ok(response),
        Err(orig) => match serde_json::from_slice::<InternalOllamaError>(line) {
            Ok(internal) => Err(OllamaError::InternalError(internal)),
            Err(_) => Err(OllamaError::JsonError(orig)),
        },
    }
}
