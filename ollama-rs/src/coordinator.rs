use crate::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponse},
        options::GenerationOptions,
        tools::ToolGroup,
    },
    history::ChatHistory,
    Ollama,
};

/// A coordinator for managing chat interactions and tool usage.
///
/// This struct is responsible for coordinating chat messages and tool
/// interactions within the Ollama service. It maintains the state of the
/// chat history, tools, and generation options.
pub struct Coordinator<C: ChatHistory, T: ToolGroup> {
    model: String,
    ollama: Ollama,
    options: GenerationOptions,
    history: C,
    tools: T,
    debug: bool,
}

impl<C: ChatHistory> Coordinator<C, ()> {
    /// Creates a new `Coordinator` instance without tools.
    ///
    /// # Arguments
    ///
    /// * `ollama` - The Ollama client instance.
    /// * `model` - The model to be used for chat interactions.
    /// * `history` - The chat history manager.
    ///
    /// # Returns
    ///
    /// A new `Coordinator` instance.
    pub fn new(ollama: Ollama, model: String, history: C) -> Self {
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

impl<C: ChatHistory, T: ToolGroup> Coordinator<C, T> {
    /// Creates a new `Coordinator` instance with tools.
    ///
    /// # Arguments
    ///
    /// * `ollama` - The Ollama client instance.
    /// * `model` - The model to be used for chat interactions.
    /// * `history` - The chat history manager.
    /// * `tools` - The tool group to be used.
    ///
    /// # Returns
    ///
    /// A new `Coordinator` instance with tools.
    pub fn new_with_tools(ollama: Ollama, model: String, history: C, tools: T) -> Self {
        Self {
            model,
            ollama,
            options: GenerationOptions::default(),
            history,
            tools,
            debug: false,
        }
    }

    /**
     * Sets the generation options for the coordinator.
     * 
     * # Arguments
     * 
     * * `options` - The generation options to be set.
     */
    pub fn options(mut self, options: GenerationOptions) -> Self {
        self.options = options;
        self
    }

    /**
     * Sets the debug mode for the coordinator.
     * 
     * # Arguments
     * 
     * * `debug` - The debug mode to be set.
     */
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /**
     * Sends a chat message to the Ollama service, with tool calls.
     * If the response contains tool calls, the functions are exucuted, appended to the chat history, and the chat is re-sent.
     * 
     * # Arguments
     * 
     * * `messages` - The chat messages to be sent.
     * 
     * # Returns
     * 
     * The response from the Ollama service. 
     *
     */
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
                &mut self.history,
                ChatMessageRequest::new(self.model.clone(), messages)
                    .options(self.options.clone())
                    .tools::<T>(),
            )
            .await?;

        if !resp.message.tool_calls.is_empty() {
            for call in resp.message.tool_calls {
                if self.debug {
                    eprintln!("Tool call: {:?}", call.function);
                }

                let resp = self.tools.call(&call.function).await?;

                if self.debug {
                    eprintln!("Tool response: {}", &resp);
                }

                self.history.push(ChatMessage::tool(resp))
            }

            // recurse
            Box::pin(self.chat(vec![])).await
        } else {
            if self.debug {
                eprintln!(
                    "Response from {} of type {:?}: '{}'",
                    resp.model, resp.message.role, resp.message.content
                );
            }

            Ok(resp)
        }
    }
}
