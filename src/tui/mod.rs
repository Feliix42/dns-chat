use crate::state::State;
use crate::transport::ChatMessage;
use crossterm::terminal;
use crossterm::ExecutableCommand;
use std::io;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

mod render;

/// An enclosing structure for the terminal backend for easy setup & teardown.
pub struct Renderer<W: Write> {
    /// The terminal backend
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> Renderer<W> {
    pub fn new(mut out: W) -> Result<Renderer<W>, crossterm::ErrorKind> {
        terminal::enable_raw_mode()?;
        out.execute(terminal::EnterAlternateScreen)?;

        Ok(Renderer {
            terminal: Terminal::new(CrosstermBackend::new(out))?,
        })
    }

    pub fn render(&mut self, state: &State) -> Result<(), crossterm::ErrorKind> {
        self.terminal
            .draw(|frame| draw(frame, state, frame.size()))?;
        Ok(())
    }
}

impl<W: Write> Drop for Renderer<W> {
    fn drop(&mut self) {
        self.terminal
            .backend_mut()
            .execute(terminal::LeaveAlternateScreen)
            .expect("Could not execute to stdout");
        terminal::disable_raw_mode().expect("Terminal doesn't support to disable raw mode");
        if std::thread::panicking() {
            eprintln!(
                "termchat paniced, to log the error you can redirect stderror to a file, example: termchat 2> termchat_log",
            );
        }
    }
}

fn draw<W: Write>(frame: &mut Frame<'_, CrosstermBackend<W>>, state: &State, size: Rect) {
    unimplemented!()
}

pub fn run(
    sender: Sender<ChatMessage>,
    recv: Receiver<ChatMessage>,
) -> Result<(), crossterm::ErrorKind> {
    let mut state = State::new();

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(6)].as_ref())
            .split(f.size());
        let block = Block::default().title("Messages").borders(Borders::ALL);
        f.render_widget(block, chunks[0]);

        let input_panel = Paragraph::new("")
            .block(Block::default().borders(Borders::ALL).title(Span::styled(
                "Compose",
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        // let block = Block::default().title("Compose").borders(Borders::ALL);
        f.render_widget(input_panel, chunks[1]);
        f.set_cursor(chunks[1].x + 1, chunks[1].y + 1)
    })?;

    loop {
        // check for input from the Message receiver
        // and hear from terminal input queue

        // call the renderer
    }

    Ok(())
}
