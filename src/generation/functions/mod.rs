pub mod pipelines;
pub mod request;
pub mod tools;

pub use crate::generation::functions::pipelines::meta_llama::request::LlamaFunctionCall;
pub use crate::generation::functions::pipelines::nous_hermes::request::NousFunctionCall;
pub use crate::generation::functions::pipelines::openai::request::OpenAIFunctionCall;
pub use crate::generation::functions::request::FunctionCallRequest;
pub use tools::Browserless;
pub use tools::DDGSearcher;
pub use tools::Scraper;
pub use tools::SerperSearchTool;
pub use tools::StockScraper;

use crate::error::OllamaError;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::RequestParserBase;
use crate::generation::functions::tools::Tool;
use std::sync::Arc;

#[cfg_attr(docsrs, doc(cfg(feature = "function-calling")))]
#[cfg(feature = "function-calling")]
impl crate::Ollama {
    fn has_system_prompt(&self, messages: &[ChatMessage], system_prompt: &str) -> bool {
        let system_message = messages.first().unwrap().clone();
        system_message.content == system_prompt
    }

    pub async fn send_function_call(
        &self,
        request: FunctionCallRequest,
        parser: Arc<dyn RequestParserBase>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let mut request = request;

        request.chat.stream = false;
        let system_prompt = parser.get_system_message(&request.tools).await;
        let model_name = request.chat.model_name.clone();

        //Make sure the first message in chat is the system prompt
        if !self.has_system_prompt(&request.chat.messages, &system_prompt.content) {
            request.chat.messages.insert(0, system_prompt);
        }
        let result = self.send_chat_messages(request.chat).await?;

        if request.raw_mode {
            return Ok(result);
        }

        let response_content: String = result.message.clone().content;
        parser
            .parse(&response_content, model_name, request.tools)
            .await
    }
}
