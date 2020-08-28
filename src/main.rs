mod app;
use app::app::App;

use crossterm::{
    event::{self, DisableMouseCapture, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::error::Error;
use std::fs;
use std::{
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

enum Event<I> {
    Input(I),
    Tick(Duration),
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(500);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
                if let CEvent::Key(KeyEvent { code, modifiers }) = event::read().unwrap() {
                    tx.send(Event::Input(KeyEvent { code, modifiers })).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick(last_tick.elapsed())).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app: App = match fs::read_to_string("db.toml") {
        Ok(db) => toml::from_str(&db).unwrap(),
        Err(_) => App::new("Todo-Timer".to_string(), terminal.get_frame().size()),
    };

    terminal.clear()?;

    loop {
        terminal.draw(|f| app.draw(f))?;
        match rx.recv()? {
            Event::Input(event) => match (event.code, event.modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;

                    fs::write("db.toml", toml::to_string(&app).unwrap())?;

                    break Ok(());
                }
                (x, modi) => {
                    app.event(x, modi);
                }
            },
            Event::Tick(duration) => {app.add_time(duration)}
            _ => {}
        };
    }
}
