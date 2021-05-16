use crate::transport::ChatMessage;
use std::iter::FromIterator;

/// Application State
///
/// Stores all sent and received messages.
#[derive(Default)]
pub struct State {
    pub messages: Vec<(ChatMessage, MessageType)>,
    pub input: Vec<char>,
    pub cursor_pos: usize,
}

#[derive(Clone)]
pub enum MessageType {
    Sent,
    Received,
}

/// Direction of cursor movement.
pub enum MoveDirection {
    Left,
    Right,
    Home,
    End,
}

impl State {
    /// Constructs a new state struct
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_received(&mut self, msg: ChatMessage) {
        self.messages.push((msg, MessageType::Received));
    }

    /// Add a character to the input string at the current cursor position.
    pub fn add_input_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    /// Bundle the input string into a message, add it to the internal storage and return it for
    /// sending.
    pub fn generate_msg(&mut self) -> Vec<ChatMessage> {
        let messages = ChatMessage::from_str(String::from_iter(self.input.iter()));
        self.messages.extend(
            messages
                .clone()
                .into_iter()
                .zip(std::iter::repeat(MessageType::Sent)),
        );
        self.input.clear();
        self.cursor_pos = 0;

        messages
    }

    /// Moves the cursor
    pub fn move_cursor(&mut self, direction: MoveDirection) {
        match direction {
            MoveDirection::Left => {
                if self.cursor_pos != 0 {
                    self.cursor_pos -= 1;
                }
            }
            MoveDirection::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }
            MoveDirection::Home => {
                self.cursor_pos = 0;
            }
            MoveDirection::End => {
                self.cursor_pos = self.input.len();
            }
        }
    }

    /// Handles a `Delete` event
    pub fn delete(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.input.remove(self.cursor_pos);
        }
    }

    /// Handles a `Backspace` event
    pub fn delete_previous(&mut self) {
        if self.cursor_pos > 0 {
            self.input.pop();
            self.cursor_pos -= 1;
        }
    }
}
