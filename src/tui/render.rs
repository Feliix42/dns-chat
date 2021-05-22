use crate::state::MessageType;
use crate::transport::ChatMessage;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

pub trait Render {
    /// Renders an element to the UI.
    fn render(self) -> Spans<'static>;
}

impl Render for (ChatMessage, MessageType) {
    fn render(self) -> Spans<'static> {
        let (msg, ty) = self;
        let sender = match ty {
            MessageType::Sent => Span::styled(
                "<You> ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            MessageType::Received => Span::styled(
                "<Them> ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        };
        let timestamp = Span::from(msg.sent.time().format("(%H:%M) ").to_string());

        Spans::from(vec![sender, timestamp, Span::from(msg.text.clone())])
    }
}
