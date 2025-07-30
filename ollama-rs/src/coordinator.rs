use std::collections::HashMap;

use crate::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponse, MessageRole},
        parameters::{FormatType, KeepAlive},
        tools::{Tool, ToolHolder, ToolInfo},
    },
    history::ChatHistory,
    models::ModelOptions,
    Ollama,
};

/// A coordinator for managing chat interactions and tool usage.
///
/// This struct is responsible for coordinating chat messages and tool
/// interactions within the Ollama service. It maintains the state of the
/// chat history, tools, and generation options.
pub struct Coordinator<C: ChatHistory> {
    model: String,
    ollama: Ollama,
    options: ModelOptions,
    history: C,
    tool_infos: Vec<ToolInfo>,
    tools: HashMap<String, Box<dyn ToolHolder>>,
    debug: bool,
    format: Option<FormatType>,
    keep_alive: Option<KeepAlive>,
    think: Option<bool>,
}

impl<C: ChatHistory> Coordinator<C> {
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
            options: ModelOptions::default(),
            history,
            tool_infos: Vec::default(),
            tools: HashMap::default(),
            debug: false,
            format: None,
            keep_alive: None,
            think: None,
        }
    }

    pub fn add_tool<T: Tool + 'static>(mut self, tool: T) -> Self {
        self.tool_infos.push(ToolInfo::new::<_, T>());
        self.tools.insert(T::name().to_string(), Box::new(tool));
        self
    }

    pub fn format(mut self, format: FormatType) -> Self {
        self.format = Some(format);
        self
    }

    pub fn options(mut self, options: ModelOptions) -> Self {
        self.options = options;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    pub fn think(mut self, think: bool) -> Self {
        self.think = Some(think);
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

        let mut request = ChatMessageRequest::new(self.model.clone(), messages)
            .options(self.options.clone())
            .tools(self.tool_infos.clone());

        if let Some(keep_alive) = &self.keep_alive {
            request = request.keep_alive(keep_alive.clone());
        }

        if let Some(think) = &self.think {
            request = request.think(*think);
        }

        if let Some(format) = &self.format {
            // If no tools are specified, set the format on the request. Otherwise wait for the
            // recursive call by checking that the last message in the history has a Tool role,
            // before setting the format. Ollama otherwise won't call the tool if the format
            // is set on the first request.
            if self.tool_infos.is_empty() {
                request = request.format(format.clone());
            } else if let Some(last_message) = self.history.messages().last() {
                if last_message.role == MessageRole::Tool {
                    request = request.format(format.clone());
                }
            }
        }

        let resp = self
            .ollama
            .send_chat_messages_with_history(&mut self.history, request)
            .await?;

        if !resp.message.tool_calls.is_empty() {
            for call in resp.message.tool_calls {
                if self.debug {
                    eprintln!("Tool call: {:?}", call.function); // TODO: Use log crate?
                }

                let Some(tool) = self.tools.get_mut(call.function.name.as_str()) else {
                    return Err(crate::error::ToolCallError::UnknownToolName.into());
                };

                let resp = tool
                    .call(call.function.arguments)
                    .await
                    .map_err(crate::error::ToolCallError::InternalToolError)?;

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
