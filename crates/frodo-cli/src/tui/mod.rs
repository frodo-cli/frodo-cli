use std::{io, time::Duration};

use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use frodo_core::tasks::{Task, TaskStatus};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Terminal,
};

/// Minimal TUI that renders tasks and allows marking them done with `d` (view-only).
/// Press `q` or `Esc` to exit.
pub fn launch(tasks: &[Task]) -> Result<()> {
    // Guard restores the terminal even if we early-return.
    let _guard = TerminalGuard::enter()?;
    let mut terminal = _guard.terminal()?;
    let mut tasks = tasks.to_owned();

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

            let items: Vec<ListItem> = tasks
                .iter()
                .map(|t| {
                    let mut line = vec![
                        Span::styled(
                            status_label(&t.status),
                            Style::default()
                                .fg(status_color(&t.status))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(&t.title, Style::default().add_modifier(Modifier::BOLD)),
                    ];
                    if let Some(desc) = &t.description {
                        line.push(Span::raw(format!(" — {desc}")));
                    }
                    ListItem::new(Line::from(line))
                })
                .collect();

            let body = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tasks (local)"),
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
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('d') => {
                        // Mark first non-done task as done (view-only for now).
                        if let Some(next) = tasks.iter_mut().find(|t| t.status != TaskStatus::Done)
                        {
                            next.status = TaskStatus::Done;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "[todo]",
        TaskStatus::InProgress => "[doing]",
        TaskStatus::Done => "[done]",
    }
}

fn status_color(status: &TaskStatus) -> Color {
    match status {
        TaskStatus::Todo => Color::Yellow,
        TaskStatus::InProgress => Color::Cyan,
        TaskStatus::Done => Color::Green,
    }
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
