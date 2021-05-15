use crate::transport::ChatMessage;

/// Application State
///
/// Stores all sent and received messages.
pub struct State {
    pub messages: Vec<(ChatMessage, MessageType)>,
}

pub enum MessageType {
    Sent,
    Received,
}

impl State {
    /// Constructs a new state struct
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}
