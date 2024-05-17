use crate::generation::chat::{ChatMessage, ChatMessageResponse};
use crate::error::OllamaError;
use crate::generation::functions::tools::Tool;
use async_trait::async_trait;
use std::sync::Arc;

pub mod openai;
pub mod nous_hermes;


#[async_trait]
pub trait RequestParserBase {
    async fn parse(&self, input: &str, model_name:String, tools: Vec<Arc<dyn Tool>>) -> Result<ChatMessageResponse, OllamaError>;
    async fn get_system_message(&self, tools: &[Arc<dyn Tool>]) -> ChatMessage;
}
