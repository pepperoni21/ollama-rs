use futures_util::StreamExt;
use futures_util::TryStreamExt;
use ollama_rs::{
    generation::completion::{
        request::GenerationRequest, GenerationContext, GenerationResponseStream,
    },
    Ollama,
};
use tokio::io::{stdout, AsyncBufReadExt, AsyncWriteExt};

const MODEL: &str = "llama2:latest";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).fuse();
    let model = args.next().unwrap_or_else(|| MODEL.to_string());

    let ollama = Ollama::default();

    let mut stdout = stdout();
    let mut commands = std::pin::pin!({
        let stdin_reader = tokio::io::BufReader::new(tokio::io::stdin());
        let line_stream = tokio_stream::wrappers::LinesStream::new(stdin_reader.lines());
        line_stream.try_take_while(|x| std::future::ready(Ok(!x.eq_ignore_ascii_case("exit"))))
    });
    let mut context: Option<GenerationContext> = None;

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;

        let input = match commands.next().await {
            Some(Ok(x)) => x,
            Some(Err(e)) => return Err(e.into()),
            None => break,
        };

        let request = {
            let mut r = GenerationRequest::new(model.clone(), input);
            r.context = context.clone();
            r
        };
        let mut stream: GenerationResponseStream = ollama.generate_stream(request).await?;

        while let Some(res) = stream.try_next().await? {
            stdout.write_all(res.response.as_bytes()).await?;
            stdout.flush().await?;
            context = res.context.or(context);
        }
    }

    Ok(())
}
