pub mod tools;
pub mod pipelines;
pub mod request;

pub use tools::WeatherTool;
pub use tools::Scraper;
pub use tools::DDGSearcher;
pub use crate::generation::functions::request::FunctionCallRequest;
pub use crate::generation::functions::pipelines::openai::request::OpenAIFunctionCall;

use crate::generation::chat::ChatMessageResponse;
use crate::generation::chat::request::{ChatMessageRequest};
use crate::generation::functions::tools::Tool;
use crate::error::OllamaError;
use std::sync::Arc;
use crate::generation::functions::pipelines::RequestParserBase;


/*impl Ollama {
    pub fn new_with_function_calling() -> Self{
        let m: Ollama = Ollama::new_default_with_history(30);
        m.add_assistant_response("default".to_string(), "openai".to_string());
        return m
    }
}*/


#[cfg(feature = "function-calling")]
impl crate::Ollama {

    pub async fn send_function_call_with_history(
        &mut self,
        request: FunctionCallRequest,
        parser: Arc<dyn RequestParserBase>,
    ) -> Result<ChatMessageResponse, OllamaError> {
        //let system_message = request.chat.messages.first().unwrap().clone();
        let system_prompt = parser.get_system_message(&request.tools).await; //TODO: Check if system prompt is added
        self.send_chat_messages_with_history(
            ChatMessageRequest::new(request.chat.model_name.clone(), vec![system_prompt.clone()]),
            "default".to_string(),
        ).await?;

        let result = self
            .send_chat_messages_with_history(
                ChatMessageRequest::new(request.chat.model_name.clone(), request.chat.messages),
                "default".to_string(),
            ).await?;

        let response_content: String = result.message.clone().unwrap().content;
        let result = parser.parse(&response_content, request.chat.model_name.clone(), request.tools).await?;
        return Ok(result);
    }
}
