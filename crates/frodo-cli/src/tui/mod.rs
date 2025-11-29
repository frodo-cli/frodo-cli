use std::{io, time::Duration};

use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

/// Minimal TUI placeholder to prove the rendering stack and input loop.
/// Press `q` or `Esc` to exit.
pub fn launch() -> Result<()> {
    // Guard restores the terminal even if we early-return.
    let _guard = TerminalGuard::enter()?;
    let mut terminal = _guard.terminal()?;

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(2),
                ])
                .split(frame.area());

            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    "Frodo",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — local-first teammate for developers"),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        "Welcome",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
                    )),
            );
            frame.render_widget(header, chunks[0]);

            let body = Paragraph::new(Line::from(vec![
                Span::raw("This is a starter view. "),
                Span::styled(
                    "Tasks, chat, and agents will live here.",
                    Style::default().fg(Color::Yellow),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Journey map (coming soon)"),
            );
            frame.render_widget(body, chunks[1]);

            let footer = Paragraph::new(Line::from(vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().fg(Color::Cyan)),
                Span::raw(" or "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" to quit."),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Controls"));
            frame.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }

    Ok(())
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        // Enter alternate screen to avoid polluting the shell buffer.
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }

    fn terminal(&self) -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        let backend = CrosstermBackend::new(io::stdout());
        Ok(Terminal::new(backend)?)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort cleanup; errors are logged but not propagated from Drop.
        if let Err(err) = disable_raw_mode() {
            eprintln!("failed to disable raw mode: {err}");
        }
        if let Err(err) = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture) {
            eprintln!("failed to restore terminal: {err}");
        }
    }
}
