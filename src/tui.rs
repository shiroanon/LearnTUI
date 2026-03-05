use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::{app::App, ui};

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

pub fn init() -> Result<Tui, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    while app.running {
        terminal.draw(|f| ui::ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Tab => {
                            app.mode = crate::app::Mode::List;
                            app.next();
                        }
                        KeyCode::BackTab => {
                            app.mode = crate::app::Mode::List;
                            app.previous();
                        }
                        KeyCode::Up | KeyCode::Char('k') => match app.mode {
                            crate::app::Mode::List => app.previous(),
                            crate::app::Mode::Content => app.scroll_up(),
                        },
                        KeyCode::Down | KeyCode::Char('j') => match app.mode {
                            crate::app::Mode::List => app.next(),
                            crate::app::Mode::Content => app.scroll_down(),
                        },
                        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_mode(),
                        KeyCode::Char('c') => app.mark_completed(),
                        _ => {}
                    }
                }
            }
        }
        app.tick();
    }
    Ok(())
}
