use crate::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponse},
        functions::ToolGroup,
        options::GenerationOptions,
    },
    history::ChatHistory,
    Ollama,
};

pub struct Coordinator<'a, 'b, C: ChatHistory, T: ToolGroup> {
    model: String,
    ollama: &'a mut Ollama,
    options: GenerationOptions,
    history: &'b mut C,
    tools: T,
    debug: bool,
}

impl<'a, 'b, C: ChatHistory> Coordinator<'a, 'b, C, ()> {
    pub fn new(ollama: &'a mut Ollama, model: String, history: &'b mut C) -> Self {
        Self {
            model,
            ollama,
            options: GenerationOptions::default(),
            history,
            tools: (),
            debug: false,
        }
    }
}

impl<'a, 'b, C: ChatHistory, T: ToolGroup> Coordinator<'a, 'b, C, T> {
    pub fn new_with_tools(
        ollama: &'a mut Ollama,
        model: String,
        history: &'b mut C,
        tools: T,
    ) -> Self {
        Self {
            model,
            ollama,
            options: GenerationOptions::default(),
            history,
            tools,
            debug: false,
        }
    }

    pub fn options(mut self, options: GenerationOptions) -> Self {
        self.options = options;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub async fn chat(
        &mut self,
        messages: Vec<ChatMessage>,
    ) -> crate::error::Result<ChatMessageResponse> {
        if self.debug {
            for m in &messages {
                eprintln!("Hit {} with:", self.model);
                eprintln!("\t{:?}: '{}'", m.role, m.content);
            }
        }

        let resp = self
            .ollama
            .send_chat_messages_with_history(
                self.history,
                ChatMessageRequest::new(self.model.clone(), messages)
                    .options(self.options.clone())
                    .tools::<T>(),
            )
            .await;

        if self.debug {
            match &resp {
                Ok(x) => eprintln!(
                    "Response from {} of type {:?}: '{}'",
                    x.model, x.message.role, x.message.content
                ),
                Err(e) => {
                    eprintln!("Error from {}: {}", self.model, e);
                }
            }
        }

        let resp = resp?;

        if !resp.message.tool_calls.is_empty() {
            for call in resp.message.tool_calls {
                let resp = self.tools.call(&call.function)?;
                self.history.push(ChatMessage::tool(resp))
            }

            // recurse
            Box::pin(self.chat(vec![])).await
        } else {
            Ok(resp)
        }
    }
}
