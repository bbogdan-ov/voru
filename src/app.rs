use std::{cmp::Ordering, io, time::Duration as Dur};

use thiserror::Error;
use tuich::{
    buffer::Buffer,
    event::Key,
    layout::{Clip, Rect},
    text::Text,
    widget::{Clear, Draw, RefDraw}
};

use crate::{
    cache::Cache,
    cmdline::CmdLine,
    commands::{CmdError, Commands},
    config::Config,
    match_keys,
    player::{PlaybackError, Player},
    server::ServerAction,
    view::{PlayerView, PlaylistsView, QueueView},
    widget::PlayerWidget,
    Action,
};

// Errors
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("Playback error: {0}")]
    Playback(PlaybackError),
    #[error("Command error: {0}")]
    Cmd(CmdError),
    #[error("Something went wrong :( : {0}")]
    Unknown(String),
}
impl From<PlaybackError> for UpdateError {
    fn from(value: PlaybackError) -> Self {
        Self::Playback(value)
    }
}
impl From<CmdError> for UpdateError {
    fn from(value: CmdError) -> Self {
        Self::Cmd(value)
    }
}

/// View kind
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Player,
    #[default]
    Playlists,
    Tracks,
    Queue,
}
impl View {
    pub fn cycle_next(&self) -> Self {
        match self {
            Self::Player => Self::Playlists,
            Self::Playlists => Self::Tracks,
            Self::Tracks => Self::Queue,
            Self::Queue => Self::Player,
        }
    }
    pub fn cycle_prev(&self) -> Self {
        match self {
            Self::Queue => Self::Tracks,
            Self::Tracks => Self::Playlists,
            Self::Playlists => Self::Player,
            Self::Player => Self::Queue
        }
    }
}

/// App mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Cmd
}

/// Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notif {
    Normal(String),
    Error(String)
}
impl Notif {
    pub fn value(&self) -> &String {
        match self {
            Self::Normal(v) => v,
            Self::Error(v) => v
        }
    }
}
impl<T: ToString> From<T> for Notif {
    fn from(value: T) -> Self {
        Self::Normal(value.to_string())
    }
}

/// State
pub struct State {
    /// Current mode
    pub mode: Mode,
    /// Current view
    pub view: View,
    /// Notification
    pub notif: Option<Notif>
}
impl State {
    pub fn enter_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn next_view(&mut self) {
        self.view = self.view.cycle_next();
    }
    pub fn prev_view(&mut self) {
        self.view = self.view.cycle_prev();
    }

    pub fn notify<N: Into<Notif>>(&mut self, notif: N) {
        self.notif = Some(notif.into());
    }
}

/// App context
pub struct AppContext {
    pub config: Config,
    pub state: State,
    pub player: Player,
    pub cache: Cache,
    pub commands: Commands
}

/// App
pub struct App {
    cmdline: CmdLine,
    player_view: PlayerView,
    playlists_view: PlaylistsView,
    queue_view: QueueView,
}
impl App {
    pub fn new() -> Self {
        Self {
            cmdline: CmdLine::new(),
            player_view: PlayerView::new(),
            playlists_view: PlaylistsView::new(),
            queue_view: QueueView::new(),
        }
    }

    fn catch_error(&self, ctx: &mut AppContext, result: Result<Action, UpdateError>) -> Action {
        match result {
            Ok(action) => action,
            Err(e) => {
                use self::UpdateError::Playback as P;
                use self::UpdateError::Cmd as C;

                // Catch an error and display it
                ctx.state.notif = match e {
                    P(PlaybackError::Io(e)) if e.kind() == io::ErrorKind::NotFound
                        => Some(Notif::Error("Couldn't play the track: No such file".into())),
                    P(PlaybackError::Play(
                        rodio::PlayError::DecoderError(
                            rodio::decoder::DecoderError::UnrecognizedFormat
                        )
                    )) => Some(Notif::Error("Couldn't play the track: Unrecognized format".into())),

                    P(PlaybackError::NoAudio) |
                    P(PlaybackError::NoTrack) |
                    P(PlaybackError::NoPlaylist) |
                    P(PlaybackError::EmptyQueue) |
                    P(PlaybackError::NotPlaying) => None,

                    P(e) => Some(Notif::Error(e.to_string())),
                    C(e) => Some(Notif::Error(e.to_string())),

                    e => Some(Notif::Error(e.to_string()))
                };

                Action::Draw
            }
        }
    }

    pub fn handle_key(
        &mut self,
        ctx: &mut AppContext,
        key: Key,
    ) -> Action {
        let result = self.try_handle_key(ctx, key);
        self.catch_error(ctx, result)
    }
    fn try_handle_key(&mut self, ctx: &mut AppContext, key: Key) -> Result<Action, UpdateError> {
        let action = if ctx.state.notif.is_some() {
            ctx.state.notif = None;
            Action::Draw
        } else {
            Action::Nope
        };

        let action = action | match ctx.state.mode {
            Mode::Normal => self.handle_normal_mode_key(ctx, key)?,
            Mode::Cmd => self.cmdline.handle_key(ctx, key)?
        };

        Ok(action)
    }

    fn handle_normal_mode_key(&mut self, ctx: &mut AppContext, key: Key) -> Result<Action, UpdateError> {
        match_keys! {
            ctx.config, key,

            enter_cmd => ctx.state.enter_mode(Mode::Cmd),

            next_view => ctx.state.next_view(),
            prev_view => ctx.state.prev_view(),

            play_next => ctx.player.play_next()?,
            play_prev => ctx.player.play_prev()?,
            replay => ctx.player.replay()?,
            resume => ctx.player.resume()?,
            pause => ctx.player.pause()?,
            stop => ctx.player.stop()?,
            toggle => ctx.player.toggle()?,
            seek_forward => ctx.player.seek_forward(Dur::from_secs(ctx.config.seek_jump))?,
            seek_backward => ctx.player.seek_backward(Dur::from_secs(ctx.config.seek_jump))?,
            volume_up => ctx.player.volume_up(ctx.config.volume_jump)?,
            volume_down => ctx.player.volume_down(ctx.config.volume_jump)?,
            volume_reset => ctx.player.set_volume(1.0)?,
            mute => ctx.player.set_muted(true)?,
            unmute => ctx.player.set_muted(false)?,
            mute_toggle => ctx.player.mute_toggle()?,

            queue_shuffle => ctx.player.queue_shuffle(),

            quit => return Ok(Action::Quit);

            else {
                return Ok(match ctx.state.view {
                    View::Tracks |
                    View::Playlists => self.playlists_view.handle_key(ctx, key)?,
                    View::Queue => self.queue_view.handle_key(ctx, key)?,
                    View::Player => Action::Nope
                })
            }
        }

        Ok(Action::Draw)
    }

    pub fn handle_server_action(
        &mut self,
        ctx: &mut AppContext,
        action: ServerAction,
    ) -> Action {
        let result = self.try_handle_server_action(ctx, action);
        self.catch_error(ctx, result)
    }
    fn try_handle_server_action(
        &mut self,
        ctx: &mut AppContext,
        action: ServerAction,
    ) -> Result<Action, UpdateError> {
        match action {
            ServerAction::Play => ctx.player.resume()?,
            ServerAction::Pause => ctx.player.pause()?,
            ServerAction::Stop => ctx.player.stop()?,
            ServerAction::PlayPause => ctx.player.toggle()?,
            ServerAction::Seek(offset) => {
                let micros = offset.as_micros();
                let dur = Dur::from_micros(micros.unsigned_abs());

                match micros.cmp(&0) {
                    Ordering::Greater => ctx.player.seek_forward(dur)?,
                    Ordering::Less => ctx.player.seek_backward(dur)?,
                    Ordering::Equal => return Ok(Action::Nope)
                }
            }
            ServerAction::Volume(vol) => ctx.player.set_volume(vol)?,

            ServerAction::Next => ctx.player.play_next()?,
            ServerAction::Prev => ctx.player.play_prev()?,
            ServerAction::Shuffle => ctx.player.queue_shuffle()
        }

        Ok(Action::Draw)
    }

    pub fn draw(
        &mut self,
        ctx: &AppContext,
        buf: &mut Buffer,
        rect: Rect,
    ) -> Rect {
        let max_width =
            if ctx.config.layout.max_width == 0 { rect.width }
            else { ctx.config.layout.max_width };
        let max_height =
            if ctx.config.layout.max_height == 0 { rect.height }
            else { ctx.config.layout.max_height };

        let rect = rect.margin((ctx.config.layout.padding_x, ctx.config.layout.padding_y));
        let rect = rect
            .min_size((max_width, max_height))
            .align_center(rect);

        // Draw player only in the playlists and queue views
        let player_rect = match ctx.state.view {
            View::Tracks |
            View::Playlists |
            View::Queue => PlayerWidget {
                ctx,
                style: ctx.config.style.player,
            }.draw(buf, rect.with_y(rect.bottom()).sub_y(2)),

            _ => Rect::default()
        };

        let view_rect = rect.margin_bottom(player_rect.height + 1);

        // Draw the views
        match ctx.state.view {
            View::Player => self.player_view.draw(ctx, buf, view_rect),
            View::Tracks |
            View::Playlists => self.playlists_view.draw(ctx, buf, view_rect),
            View::Queue => self.queue_view.draw(ctx, buf, view_rect)
        };

        // Draw error message
        if let Some(notif) = &ctx.state.notif {
            // Place message at the top
            let error_rect = rect
                .with_height(1);

            let style = match notif {
                Notif::Normal(_) => ctx.config.theme.notif_normal,
                Notif::Error(_) => ctx.config.theme.notif_error,
            };

            Clear::new(style)
                .draw(buf, error_rect);
            Text::new(notif.value(), ())
                .clip(Clip::Ellipsis)
                .draw(buf, error_rect.margin((1, 0)));
        }

        // Draw command line at the top
        if ctx.state.mode == Mode::Cmd {
            self.cmdline.draw(ctx, buf, rect);
        }

        rect
    }
}
