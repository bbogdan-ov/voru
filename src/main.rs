mod app;
mod cache;
mod config;
mod keys;
mod player;
mod playlist;
mod track;
mod traits;
mod view;
mod widget;
mod cmdline;
mod commands;
mod cli;
mod server;

use std::{io::{self, Read}, ops::BitOr, sync::mpsc, thread};

use app::{App, AppContext, Mode, State, View};
use cache::Cache;
use cli::{print_help, print_version, Cli};
use commands::Commands;
use config::{default_config_path, Config, ConfigError};
use player::Player;
use playlist::{playlists_form_config, LoadPlaylistsError};
use rodio::OutputStream;
use server::ServerAction;
use thiserror::Error;
use tuich::{backend::{crossterm::CrosstermBackend, BackendEvent, BackendEventReader}, event::Event, terminal::Terminal};
use widget::ListEvent;

// Errors
#[derive(Debug, Error)]
enum AppError {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("Environment variable error: {0}")]
    Var(std::env::VarError),
    #[error("Config error: {0}")]
    Config(ConfigError),
    #[error("Load playlists error: {0}")]
    LoadPlaylists(LoadPlaylistsError),
    #[error("Audio stream error: {0}")]
    AudioStream(rodio::StreamError),
    #[error("Zbus error: {0}")]
    Zbus(mpris_server::zbus::Error)
}
impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<std::env::VarError> for AppError {
    fn from(value: std::env::VarError) -> Self {
        Self::Var(value)
    }
}

// Consts
const TICK_INTERVAL: u64 = 500;

// Types
pub type Term = Terminal<CrosstermBackend<io::Stdout>>;

/// Update kind
#[derive(Debug, Clone, PartialEq)]
enum UpdateKind {
    Tick,
    Event(Event),
    Server(ServerAction)
}

/// App action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Nope,
    Draw,
    Resize(u16, u16),
    Quit
}
impl BitOr for Action {
    type Output = Action;
    fn bitor(self, rhs: Self) -> Self::Output {
        if self == Action::Nope { rhs }
        else { self }
    }
}
impl From<ListEvent> for Action {
    fn from(value: ListEvent) -> Self {
        match value {
            ListEvent::Nope => Self::Nope,
            _ => Self::Draw
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), AppError> {
    // Set panic hook
    std::panic::set_hook(Box::new(|msg| {
        let mut stdout = io::stdout();

        crossterm::terminal::disable_raw_mode().unwrap();
        crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
        eprintln!("VORU panicked: {}", msg);
    }));

    // Init commands
    let commands = Commands::new();

    // Parse args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cli) = Cli::parse(&args) else {
        // Print help message and exit if an unknown arg was given
        print_help(&commands);
        std::process::exit(1);
    };

    if cli.print_help {
        // Print help message and exit
        print_help(&commands);
        return Ok(())
    } else if cli.print_version {
        // Print current version and exit
        print_version();
        return Ok(())
    }

    // Trying to load a config
    let config_path = cli.config_path.unwrap_or(default_config_path().map_err(AppError::Config)?);
    let config = match Config::from_path(&config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Unable to load the config {:?}:", config_path);
            eprintln!("{}\n", e);
            eprintln!("Press <Enter> to continue with the default config...");
            let _ = io::stdin().read(&mut []);

            Config::default()
        }
    };

    let (sender, receiver) = mpsc::channel::<UpdateKind>();

    // Init audio stream
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(AppError::AudioStream)?;

    // Init cache
    let mut cache = Cache::new();

    // Load playlists
    let playlists = playlists_form_config(&mut cache, &config)
        .map_err(AppError::LoadPlaylists)?;
    let player = Player::new(stream_handle, playlists, sender.clone()).await?;

    // Init state
    let mut state = State {
        mode: Mode::Normal,
        view: View::default(),
        notif: None
    };

    // Echo on startup
    if let Some(echo_msg) = cli.echo_msg {
        state.notify(echo_msg);
    }

    // Init app context
    let mut ctx = AppContext {
        config,
        state,
        player,
        cache,
        commands
    };

    // Init terminal
    let mut term: Term = Terminal::classic(CrosstermBackend::default())?;
    // Init app
    let mut app = App::new();

    // Handle events
    handle_events(&term, sender.clone());
    handle_tick(sender.clone());

    draw(&ctx, &mut term, &mut app)?;

    loop {
        let action = match receiver.recv() {
            Ok(UpdateKind::Tick) => {
                ctx.player.handle_tick();
                Action::Draw
            }
            Ok(UpdateKind::Event(event)) => {
                match event {
                    Event::Key(key, _) => {
                        app.handle_key(&mut ctx, key)
                    },
                    Event::Resize(w, h) => Action::Resize(w, h),
                    _ => Action::Nope
                }
            }
            Ok(UpdateKind::Server(action)) => app.handle_server_action(&mut ctx, action),
            Err(_) => Action::Nope
        };

        match action {
            Action::Nope => continue,
            Action::Draw => (),
            Action::Resize(w, h) => term.resize(w, h)?,
            Action::Quit => break Ok(())
        }

        draw(&ctx, &mut term, &mut app)?;
    }
}
fn draw(ctx: &AppContext, term: &mut Term, app: &mut App) -> io::Result<()> {
    let rect = term.rect();

    term.clear();
    app.draw(
        ctx,
        &mut term.buffer,
        rect,
    );
    term.draw()
}

//
fn handle_events(term: &Term, sender: mpsc::Sender<UpdateKind>) {
    let event_reader = term.event_reader();

    thread::spawn(move || -> io::Result<()> {
        let mut event_reader = event_reader.clone();

        loop {
            let event = event_reader.read_events()?;
            let _ = sender.send(UpdateKind::Event(event));
        }
    });
}
fn handle_tick(sender: mpsc::Sender<UpdateKind>) {
    thread::spawn(move || {
        loop {
            let _ = sender.send(UpdateKind::Tick);
            thread::sleep(std::time::Duration::from_millis(TICK_INTERVAL));
        }
    });
}
