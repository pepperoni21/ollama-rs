use std::borrow::Cow;

use crate::generation::chat::ChatMessage;

/// A trait for managing chat message history.
///
/// This trait provides methods for adding messages to the history and
/// retrieving the list of messages.
pub trait ChatHistory {
    /// Adds a chat message to the history.
    ///
    /// # Arguments
    ///
    /// * `message` - The chat message to be added.
    fn push(&mut self, message: ChatMessage);
    /// Returns a reference to the list of chat messages in the history.
    ///
    /// The messages are returned as a `Cow` (Clone on Write) to allow for
    /// efficient borrowing or cloning as needed.
    fn messages(&self) -> Cow<'_, [ChatMessage]>;
}

impl ChatHistory for Vec<ChatMessage> {
    /// Adds a chat message to the history.
    ///
    /// # Arguments
    ///
    /// * `message` - The chat message to be added.
    fn push(&mut self, message: ChatMessage) {
        self.push(message);
    }

    /// Returns a reference to the list of chat messages in the history.
    ///
    /// The messages are returned as a `Cow` (Clone on Write) to allow for
    /// efficient borrowing or cloning as needed.
    fn messages(&self) -> Cow<'_, [ChatMessage]> {
        Cow::Borrowed(self)
    }
}
