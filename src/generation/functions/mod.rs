pub mod pipelines;
pub mod request;
pub mod tools;

pub use crate::generation::functions::pipelines::openai::request::OpenAIFunctionCall;
pub use crate::generation::functions::request::FunctionCallRequest;
pub use tools::DDGSearcher;
pub use tools::Scraper;

use crate::error::OllamaError;
use crate::generation::chat::request::ChatMessageRequest;
use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::generation::functions::pipelines::RequestParserBase;
use crate::generation::functions::tools::Tool;
use std::sync::Arc;

#[cfg(feature = "function-calling")]
impl crate::Ollama {
    pub async fn check_system_message(
        &self,
        messages: &Vec<ChatMessage>,
        system_prompt: &str,
    ) -> bool {
        let system_message = messages.first().unwrap().clone();
        return system_message.content == system_prompt;
    }

    #[cfg(feature = "chat-history")]
    pub async fn send_function_call_with_history(
        &mut self,
        request: FunctionCallRequest,
        parser: Arc<dyn RequestParserBase>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        let system_prompt = parser.get_system_message(&request.tools).await;
        if request.chat.messages.len() == 0 {
            // If there are no messages in the chat, add a system prompt
            self.send_chat_messages_with_history(
                ChatMessageRequest::new(
                    request.chat.model_name.clone(),
                    vec![system_prompt.clone()],
                ),
                "default".to_string(),
            )
            .await?;
        }

        let result = self
            .send_chat_messages_with_history(
                ChatMessageRequest::new(request.chat.model_name.clone(), request.chat.messages),
                "default".to_string(),
            )
            .await?;

        let response_content: String = result.message.clone().unwrap().content;

        let result = parser
            .parse(
                &response_content,
                request.chat.model_name.clone(),
                request.tools,
            )
            .await?;
        return Ok(result);
    }

    pub async fn send_function_call(
        &self,
        request: FunctionCallRequest,
        parser: Arc<dyn RequestParserBase>,
    ) -> crate::error::Result<ChatMessageResponse> {
        let mut request = request;

        request.chat.stream = false;
        let system_prompt = parser.get_system_message(&request.tools).await;
        let model_name = request.chat.model_name.clone();

        //Make sure the first message in chat is the system prompt
        if !self
            .check_system_message(&request.chat.messages, &system_prompt.content)
            .await
        {
            request.chat.messages.insert(0, system_prompt);
        }
        let result = self.send_chat_messages(request.chat).await?;
        let response_content: String = result.message.clone().unwrap().content;

        let result = parser
            .parse(&response_content, model_name, request.tools)
            .await?;
        return Ok(result);
    }
}
