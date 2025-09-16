use bytes::{Bytes, BytesMut};
use futures_util::Stream;
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    pub struct Lines<S> {
        #[pin]
        inner: S,
        buf: BytesMut,
    }
}

/// Convert a `Stream<Item=Bytes>` into a `Stream<Item=Bytes>` of lines.
/// Supports both `\n` and `\r\n`. Returned lines exclude the delimiter(s).
pub fn lines<S, E>(stream: S) -> Lines<S>
where
    S: Stream<Item = Result<Bytes, E>>,
{
    Lines {
        inner: stream,
        buf: BytesMut::new(),
    }
}

impl<S, E> Stream for Lines<S>
where
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = Result<Bytes, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            if let Some(pos) = this.buf.iter().position(|&b| b == b'\n') {
                // Detect CRLF safely (avoid [pos-1] panic)
                let is_crlf = matches!(pos.checked_sub(1), Some(p) if this.buf[p] == b'\r');
                let line_end = if is_crlf { pos - 1 } else { pos };

                // split including the delimiter
                let mut line = this.buf.split_to(pos + 1);
                // truncate to exclude delimiter(s)
                line.truncate(line_end);

                return Poll::Ready(Some(Ok(line.freeze())));
            }

            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(chunk)) => match chunk {
                    Ok(chunk) => {
                        this.buf.extend_from_slice(&chunk);
                    }
                    Err(e) => {
                        return Poll::Ready(Some(Err(e)));
                    }
                },
                Poll::Ready(None) => {
                    if !this.buf.is_empty() {
                        let rest = this.buf.split().freeze();
                        return Poll::Ready(Some(Ok(rest)));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures_util::stream;
    use tokio_stream::StreamExt;

    async fn collect_lines<'a>(chunks: Vec<&'a [u8]>) -> Vec<String> {
        let s = stream::iter(
            chunks
                .into_iter()
                .map(|c| Result::<_, ()>::Ok(Bytes::copy_from_slice(c))),
        );
        lines(s)
            .map(|b| String::from_utf8(b.unwrap().to_vec()).unwrap())
            .collect::<Vec<_>>()
            .await
    }

    #[tokio::test]
    async fn test_simple_crlf() {
        let lines_out = collect_lines(vec![b"hello\r\nworld\r\n"]).await;
        assert_eq!(lines_out, vec!["hello", "world"]);
    }

    #[tokio::test]
    async fn test_simple_lf() {
        let lines_out = collect_lines(vec![b"foo\nbar\n"]).await;
        assert_eq!(lines_out, vec!["foo", "bar"]);
    }

    #[tokio::test]
    async fn test_mixed_delimiters() {
        let lines_out = collect_lines(vec![b"abc\r\ndef\nxyz\r\n"]).await;
        assert_eq!(lines_out, vec!["abc", "def", "xyz"]);
    }

    #[tokio::test]
    async fn test_split_across_chunks() {
        let lines_out = collect_lines(vec![b"hel", b"lo\r", b"\nwor", b"ld\n"]).await;
        assert_eq!(lines_out, vec!["hello", "world"]);
    }

    #[tokio::test]
    async fn test_trailing_without_newline() {
        let lines_out = collect_lines(vec![b"no newline"]).await;
        assert_eq!(lines_out, vec!["no newline"]);
    }

    #[tokio::test]
    async fn test_leading_newline() {
        let lines_out = collect_lines(vec![b"\nhello\nworld\n"]).await;
        assert_eq!(lines_out, vec!["", "hello", "world"]);
    }

    #[tokio::test]
    async fn test_leading_crlf() {
        let lines_out = collect_lines(vec![b"\r\nfoo\r\nbar\r\n"]).await;
        assert_eq!(lines_out, vec!["", "foo", "bar"]);
    }

    #[tokio::test]
    async fn test_multiple_empty_lines() {
        let lines_out = collect_lines(vec![b"\n\n\n"]).await;
        assert_eq!(lines_out, vec!["", "", ""]);
    }

    #[tokio::test]
    async fn test_crlf_and_lf_mix_across_chunks() {
        let lines_out = collect_lines(vec![b"line1\r", b"\nline2\nline", b"3\r\n"]).await;
        assert_eq!(lines_out, vec!["line1", "line2", "line3"]);
    }

    #[tokio::test]
    async fn test_json_linebreaks_pass_through() {
        let original = "a\nb";
        let json = serde_json::to_vec_pretty(original).unwrap();
        let lines_out = collect_lines(vec![&json]).await;
        assert_eq!(&lines_out, &[r#""a\nb""#]);
        assert_eq!(
            original,
            serde_json::from_str::<String>(&lines_out[0]).unwrap()
        );
    }

    #[tokio::test]
    async fn test_propagates_error() {
        #[derive(Debug, PartialEq)]
        struct MyError;
        let s = stream::iter(vec![
            Ok(Bytes::from_static(b"hello\n")),
            Err(MyError),
            Ok(Bytes::from_static(b"world\n")),
        ]);

        let out: Vec<Result<String, MyError>> = lines(s)
            .map(|res| res.map(|b| String::from_utf8(b.to_vec()).unwrap()))
            .collect()
            .await;

        assert_eq!(
            out,
            vec![Ok("hello".into()), Err(MyError), Ok("world".into())]
        );
    }
}
