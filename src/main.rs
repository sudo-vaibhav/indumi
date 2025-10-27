mod editor;
mod ui;

// Use library modules
use indumi::{calc, parser, currency};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use crate::editor::Editor;
use crate::calc::Calculator;
use crate::ui::render_ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create calculator with currency rates
    let calculator = Calculator::new().await?;

    // Create editor state
    let mut editor = Editor::new(calculator);

    // Main loop
    loop {
        terminal.draw(|f| {
            render_ui(f, &editor);
        })?;

        if let Event::Key(key) = event::read()? {
            if should_quit(&key) {
                break;
            }
            editor.handle_key(key);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn should_quit(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}
