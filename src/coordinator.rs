use crate::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponse},
        options::GenerationOptions,
    },
    history::ChatHistory,
    Ollama,
};

pub struct Coordinator<'a, 'b, C: ChatHistory> {
    model: String, // TODO this should not require ownership
    ollama: &'a mut Ollama,
    options: GenerationOptions,
    history: &'b mut C,
    debug: bool,
}

impl<'a, 'b, C: ChatHistory> Coordinator<'a, 'b, C> {
    pub fn new(ollama: &'a mut Ollama, history: &'b mut C, model: String) -> Self {
        Self {
            model,
            ollama,
            options: GenerationOptions::default(),
            history,
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
        message: ChatMessage,
    ) -> crate::error::Result<ChatMessageResponse> {
        if self.debug {
            eprintln!(
                "Hit {} with {:?}: '{}'",
                self.model, message.role, message.content
            );
        }

        let resp = self
            .ollama
            .send_chat_messages_with_history(
                self.history,
                ChatMessageRequest::new(self.model.clone(), vec![message])
                    .options(self.options.clone()),
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

        resp
    }
}
