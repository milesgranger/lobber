mod ai;
mod game;
mod physics;
mod render;
mod terrain;

use std::io;
use std::panic;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use render::app::App;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Panic handler to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    // Run the game
    let result = run_game(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Game error: {e}");
    }

    Ok(())
}

fn run_game(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    // Show title screen
    show_title_screen(terminal)?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        app.handle_input();
        app.update();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

const TITLE_ART: &str = r#"
    __    ____  ____  ____  __________
   / /   / __ \/ __ )/ __ )/ ____/ __ \
  / /   / / / / __  / __  / __/ / /_/ /
 / /___/ /_/ / /_/ / /_/ / /___/ _, _/
/_____/\____/_____/_____/_____/_/ |_|

"#;

fn show_title_screen(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let text = format!(
                "{}\n\
                 \n\
                 A TUI Artillery Game\n\
                 \n\
                 Inspired by Scorched Earth (1991)\n\
                 \n\
                 Controls:\n\
                   h/Left  l/Right  -  Adjust angle\n\
                   k/Up    j/Down   -  Adjust power\n\
                   a/d              -  Move tank\n\
                   Tab              -  Switch ammo\n\
                   Space / Enter    -  Fire!\n\
                   Q                -  Quit\n\
                 \n\
                 Press any key to start...",
                TITLE_ART,
            );

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    break;
                }
            }
        }
    }

    Ok(())
}
