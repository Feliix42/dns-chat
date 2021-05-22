use crate::state::{MoveDirection, State};
use crate::transport::ChatMessage;
use crate::tui::render::Render;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal, ExecutableCommand,
};
use std::{
    io::{self, Write},
    iter::FromIterator,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
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
    }
}

fn draw<W: Write>(frame: &mut Frame<'_, CrosstermBackend<W>>, state: &State, size: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(6)].as_ref())
        .split(size);

    let block = Block::default().title("Messages").borders(Borders::ALL);
    let messages: Vec<Spans> = state
        .messages
        .clone()
        .into_iter()
        .map(|msg| msg.render())
        .collect();
    let message_panel = Paragraph::new(messages)
        .block(block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(message_panel, chunks[0]);

    let input_panel = Paragraph::new(String::from_iter(state.input.iter()))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            "Compose",
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
    frame.render_widget(input_panel, chunks[1]);
    let inner_width = chunks[1].width - 2;
    frame.set_cursor(
        chunks[1].x + 1 + (state.cursor_pos as u16 % inner_width),
        chunks[1].y + 1 + (state.cursor_pos as u16 / inner_width),
    );
}

pub fn run(
    sender: Sender<ChatMessage>,
    recv: Receiver<ChatMessage>,
) -> Result<(), crossterm::ErrorKind> {
    let mut state = State::new();

    let stdout = io::stdout();
    let mut renderer = Renderer::new(stdout)?;

    'main: loop {
        // check for input from the Message receiver
        if let Ok(msg) = recv.try_recv() {
            state.add_received(msg);
        }

        // and hear from terminal input queue
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Mouse(_) => (),
                Event::Resize(_, _) => (),
                Event::Key(KeyEvent { code, modifiers }) => match code {
                    KeyCode::Char(character) => {
                        if character == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                            break 'main;
                        } else {
                            state.add_input_char(character);
                        }
                    }
                    KeyCode::Enter => {
                        for msg in state.generate_msg() {
                            sender
                                .send(msg)
                                .expect("Message sender thread unavailable!");
                        }
                    }
                    KeyCode::Delete => {
                        state.delete();
                    }
                    KeyCode::Backspace => {
                        state.delete_previous();
                    }
                    KeyCode::Left => {
                        state.move_cursor(MoveDirection::Left);
                    }
                    KeyCode::Right => {
                        state.move_cursor(MoveDirection::Right);
                    }
                    KeyCode::Home => {
                        state.move_cursor(MoveDirection::Home);
                    }
                    KeyCode::End => {
                        state.move_cursor(MoveDirection::End);
                    }
                    KeyCode::Esc => {
                        break 'main;
                    }
                    _ => (),
                },
            }
        }

        // call the renderer
        renderer.render(&state)?;
    }

    Ok(())
}
