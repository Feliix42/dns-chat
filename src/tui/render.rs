use crate::transport::ChatMessage;
use tui::widgets::{Paragraph, Widget};

pub trait Render<T: Widget> {
    /// Renders an element to the UI.
    ///
    /// `width` denotes the maximum line width.
    fn render(&self, width: usize) -> T;
}

impl<Paragraph: Widget> Render<Paragraph> for ChatMessage {
    fn render(&self, width: usize) -> Paragraph {
        unimplemented!()
    }
}
