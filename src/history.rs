use std::borrow::Cow;

use crate::generation::chat::ChatMessage;

pub trait ChatHistory {
    fn push(&mut self, message: ChatMessage);
    fn messages(&self) -> Cow<'_, [ChatMessage]>;
}

impl ChatHistory for Vec<ChatMessage> {
    fn push(&mut self, message: ChatMessage) {
        self.push(message);
    }

    fn messages(&self) -> Cow<'_, [ChatMessage]> {
        Cow::Borrowed(self)
    }
}
