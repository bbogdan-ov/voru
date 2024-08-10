use std::{collections::HashMap, path::PathBuf, rc::Rc, time::Duration};

use thiserror::Error;

use crate::{app::{State, UpdateError}, cache::Cache, player::{Player, QueueTrack}, track::Track, traits::Expand, Action};

// Errors
#[derive(Debug, Error)]
pub enum CmdError {
    #[error("No such command")]
    NoSuchCmd,
    #[error("Not enough arguments")]
    NotEnoughArgs,
    #[error("Invalid argument type \"{0}\"")]
    InvalidArg(String),
    #[error("No such file or directory \"{0}\"")]
    NoSuchFile(PathBuf)
}

/// Command kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmdKind {
    Quit,
    Hello,
    Echo,

    PlayNext,
    PlayPrev,
    Replay,
    Resume,
    Pause,
    Stop,
    Toggle,
    Seek,
    SeekForward,
    SeekBackward,
    Volume,
    VolumeUp,
    VolumeDown,
    VolumeReset,
    Mute,
    Unmute,
    MuteToggle,

    QueueAdd,
    QueueClear,
    QueueShuffle,
}
impl CmdKind {
    pub fn args(&self) -> Option<&'static str> {
        Some(match self {
            Self::Echo => "<MSG>",

            Self::Seek => "<SECONDS>",
            Self::SeekForward => "<SECONDS>",
            Self::SeekBackward => "<SECONDS>",
            Self::Volume => "<PERCENTAGE>",
            Self::VolumeUp => "<PERCENTAGE>",
            Self::VolumeDown => "<PERCENTAGE>",

            Self::QueueAdd => "<TRACKS>",

            _ => return None
        })
    }
    pub fn description(&self) -> &'static str {
        match self {
            Self::Quit => "Say \"goodbye\" to VORU",
            Self::Hello => "Say \"hello\" to VORU!",
            Self::Echo => "Say something else",

            Self::PlayNext => "Play next track in the queue",
            Self::PlayPrev => "Play previous track in the queue",
            Self::Replay => "Play the first track in the queue",
            Self::Resume => "Resume playback",
            Self::Pause => "Pause playback",
            Self::Stop => "Stop playback",
            Self::Toggle => "Resume/pause playback",
            Self::Seek => "Seek to <SECONDS>",
            Self::SeekForward => "Seek forward by <SECONDS>",
            Self::SeekBackward => "Seek backward by <SECONDS>",
            Self::Volume => "Set volume to <PERCENTAGE>",
            Self::VolumeUp => "Increase volume by <PERCENTAGE>",
            Self::VolumeDown => "Decrease volume by <PERCENTAGE>",
            Self::VolumeReset => "Reset volume to 100%",
            Self::Mute => "Mute audio",
            Self::Unmute => "Unmute audio",
            Self::MuteToggle => "Mute/unmute audio",

            Self::QueueAdd => "Add <TRACKS> to the queue",
            Self::QueueClear => "Clear the queue",
            Self::QueueShuffle => "Randomize order of the queue"
        }
    }
}

/// Command
#[derive(Debug)]
pub enum Cmd {
    Normal(CmdKind),
    Alias(CmdKind, &'static str)
}
impl Cmd {
    pub fn kind(&self) -> &CmdKind {
        match self {
            Self::Normal(kind) => kind,
            Self::Alias(kind, _) => kind
        }
    }
    pub fn is_alias(&self) -> bool {
        match self {
            Self::Alias(_, _) => true,
            _ => false
        }
    }
}

/// Commands
#[derive(Debug)]
pub struct Commands {
    pub list: HashMap<&'static str, Cmd>,
}
impl Commands {
    pub fn new() -> Self {
        let list = HashMap::from([
            ("quit",          Cmd::Normal(CmdKind::Quit)),
            ("q",             Cmd::Alias(CmdKind::Quit, "quit")),
            ("bye",           Cmd::Alias(CmdKind::Quit, "quit")),
            ("hello",         Cmd::Normal(CmdKind::Hello)),
            ("echo",          Cmd::Normal(CmdKind::Echo)),

            ("play-next",     Cmd::Normal(CmdKind::PlayNext)),
            ("next",          Cmd::Alias(CmdKind::PlayNext, "play-next")),
            ("play-prev",     Cmd::Normal(CmdKind::PlayPrev)),
            ("prev",          Cmd::Alias(CmdKind::PlayPrev, "play-prev")),
            ("replay",        Cmd::Normal(CmdKind::Replay)),
            ("resume",        Cmd::Normal(CmdKind::Resume)),
            ("pause",         Cmd::Normal(CmdKind::Pause)),
            ("stop",          Cmd::Normal(CmdKind::Stop)),
            ("toggle",        Cmd::Normal(CmdKind::Toggle)),
            ("seek",          Cmd::Normal(CmdKind::Seek)),
            ("seek-forw",     Cmd::Normal(CmdKind::SeekForward)),
            ("seekf",         Cmd::Alias(CmdKind::SeekForward, "seek-forw")),
            ("seek-back",     Cmd::Normal(CmdKind::SeekBackward)),
            ("seekb",         Cmd::Alias(CmdKind::SeekBackward, "seek-back")),
            ("volume",        Cmd::Normal(CmdKind::Volume)),
            ("vol",           Cmd::Alias(CmdKind::Volume, "volume")),
            ("volume-up",     Cmd::Normal(CmdKind::VolumeUp)),
            ("volup",         Cmd::Alias(CmdKind::VolumeUp, "volume-up")),
            ("volume-down",   Cmd::Normal(CmdKind::VolumeDown)),
            ("voldown",       Cmd::Alias(CmdKind::VolumeDown, "volume-down")),
            ("volume-reset",  Cmd::Normal(CmdKind::VolumeReset)),
            ("volreset",      Cmd::Alias(CmdKind::VolumeReset, "volume-reset")),
            ("mute",          Cmd::Normal(CmdKind::Mute)),
            ("unmute",        Cmd::Normal(CmdKind::Unmute)),
            ("mute-toggle",   Cmd::Normal(CmdKind::MuteToggle)),
            ("mutetog",       Cmd::Alias(CmdKind::MuteToggle, "mute-toggle")),

            ("queue-add",     Cmd::Normal(CmdKind::QueueAdd)),
            ("add",           Cmd::Alias(CmdKind::QueueAdd, "queue-add")),
            ("queue-clear",   Cmd::Normal(CmdKind::QueueClear)),
            ("clear",         Cmd::Alias(CmdKind::QueueClear, "queue-clear")),
            ("queue-shuffle", Cmd::Normal(CmdKind::QueueShuffle)),
            ("shuffle",       Cmd::Alias(CmdKind::QueueShuffle, "queue-shuffle")),
        ]);

        Self {
            list
        }
    }

    /// Execute command with args by given string
    /// For example: `"queue-add ~/my-cool-music-dir/*"`
    pub fn exec<S: AsRef<str>>(&self, cache: &mut Cache, state: &mut State, player: &mut Player, command: S) -> Result<Action, UpdateError> {
        let command = command.as_ref().trim();
        let (cmd_str, args_str) = match command.split_once(' ') {
            Some((cmd, args)) => (cmd, args.trim()),
            None => (command, "")
        };
        let args: Vec<&str> = args_str
            .split(' ')
            .filter(|a| !a.is_empty())
            .collect();

        let first_arg = args.get(0);

        let cmd = self.list.get(cmd_str)
            .ok_or(CmdError::NoSuchCmd)?;

        match cmd.kind() {
            CmdKind::Quit => return Ok(Action::Quit),
            CmdKind::Hello => state.notify("hey"),
            CmdKind::Echo => state.notify(args_str),

            CmdKind::PlayNext => player.play_next()?,
            CmdKind::PlayPrev => player.play_prev()?,
            CmdKind::Replay => player.replay()?,
            CmdKind::Resume => player.resume()?,
            CmdKind::Pause => player.pause()?,
            CmdKind::Stop => player.stop()?,
            CmdKind::Toggle => player.toggle()?,
            CmdKind::Seek => player.seek(parse_secs(args.get(0))?)?,
            CmdKind::SeekForward => player.seek_forward(parse_secs(first_arg)?)?,
            CmdKind::SeekBackward => player.seek_backward(parse_secs(first_arg)?)?,
            CmdKind::Volume => player.set_volume(parse_percent(first_arg)?)?,
            CmdKind::VolumeUp => player.volume_up(parse_percent(first_arg)?)?,
            CmdKind::VolumeDown => player.volume_down(parse_percent(first_arg)?)?,
            CmdKind::VolumeReset => player.set_volume(1.0)?,
            CmdKind::Mute => player.set_muted(true)?,
            CmdKind::Unmute => player.set_muted(false)?,
            CmdKind::MuteToggle => player.mute_toggle()?,

            CmdKind::QueueAdd => cmd_add(cache, state, player, args)?,
            CmdKind::QueueClear => player.queue_clear()?,
            CmdKind::QueueShuffle => player.queue_shuffle(),
        }

        Ok(Action::Draw)
    }
}

fn cmd_add(cache: &mut Cache, state: &mut State, player: &mut Player, args: Vec<&str>) -> Result<(), UpdateError> {
    if args.is_empty() {
        return Err(CmdError::NotEnoughArgs.into());
    }

    let mut tracks = vec![];

    for arg in args {
        let path = arg.expand()
            .map_err(|e| UpdateError::Unknown(e.to_string()))?;
        let paths = path.expand_to_multiple()
            .map_err(UpdateError::Io)?;

        for path in paths {
            if !path.exists() {
                return Err(CmdError::NoSuchFile(path).into());
            }
            if !path.is_file() { continue }
            let Ok(track) = Track::from_path(cache, path) else {
                continue;
            };

            tracks.push(Rc::new(QueueTrack::Signle(Rc::new(track))));
        }
    }

    state.notify(format!("{} tracks were added", tracks.len()));
    player.queue_add_tracks(tracks);
    Ok(())
}

fn parse_secs<S: AsRef<str>>(arg: Option<S>) -> Result<Duration, CmdError> {
    let arg = arg.ok_or(CmdError::NotEnoughArgs)?;
    let arg = arg.as_ref();
    let secs = arg.parse::<u64>().map_err(|_| CmdError::InvalidArg(arg.to_string()))?;

    Ok(Duration::from_secs(secs))
}
fn parse_percent<S: AsRef<str>>(arg: Option<S>) -> Result<f32, CmdError> {
    let arg = arg.ok_or(CmdError::NotEnoughArgs)?;
    let arg = arg.as_ref().trim_end_matches('%');
    let percent = arg.parse::<u16>()
        .map_err(|_| CmdError::InvalidArg(arg.to_string()))?;

    Ok(percent as f32 / 100.0)
}
