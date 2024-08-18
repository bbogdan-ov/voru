use std::{path::PathBuf, rc::Rc, time::Duration};

use thiserror::Error;

use crate::{app::{AppContext, UpdateError}, player::{LoopState, QueueTrack}, track::Track, traits::Expand, Action};

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
    LoopNone,
    LoopQueue,
    LoopShuffle,

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
            Self::Resume => "Resume playback or replay the current track",
            Self::Pause => "Pause playback",
            Self::Stop => "Stop playback and clear currently playing track",
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
            Self::LoopNone => "Disable looping",
            Self::LoopQueue => "Repeat the queue after the end",
            Self::LoopShuffle => "Shuffle and repeat the queue after the end",

            Self::QueueAdd => "Add <TRACKS> to the queue",
            Self::QueueClear => "Clear the queue",
            Self::QueueShuffle => "Randomize order of the queue"
        }
    }
}

/// Command
#[derive(Debug)]
pub enum Cmd {
    Normal(&'static str, CmdKind),
    Alias(&'static str, CmdKind, &'static str)
}
impl Cmd {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal(name, _) => name,
            Self::Alias(name, _, _) => name
        }
    }
    pub fn kind(&self) -> &CmdKind {
        match self {
            Self::Normal(_, kind) => kind,
            Self::Alias(_, kind, _) => kind
        }
    }
    pub fn is_alias(&self) -> bool {
        matches!(self, Self::Alias(_, _, _))
    }
}

/// Commands
#[derive(Debug)]
pub struct Commands {
    pub list: [Cmd; 40]
}
impl Commands {
    pub fn new() -> Self {
        Self { list: [
            Cmd::Normal("quit", CmdKind::Quit),
            Cmd::Alias("q", CmdKind::Quit, "quit"),
            Cmd::Alias("bye", CmdKind::Quit, "quit"),
            Cmd::Normal("hello", CmdKind::Hello),
            Cmd::Normal("echo", CmdKind::Echo),

            Cmd::Normal("play-next", CmdKind::PlayNext),
            Cmd::Alias("next", CmdKind::PlayNext, "play-next"),
            Cmd::Normal("play-prev", CmdKind::PlayPrev),
            Cmd::Alias("prev", CmdKind::PlayPrev, "play-prev"),
            Cmd::Normal("replay", CmdKind::Replay),
            Cmd::Normal("resume", CmdKind::Resume),
            Cmd::Normal("pause", CmdKind::Pause),
            Cmd::Normal("stop", CmdKind::Stop),
            Cmd::Normal("toggle", CmdKind::Toggle),
            Cmd::Normal("seek", CmdKind::Seek),
            Cmd::Normal("seek-forw", CmdKind::SeekForward),
            Cmd::Alias("seekf", CmdKind::SeekForward, "seek-forw"),
            Cmd::Normal("seek-back", CmdKind::SeekBackward),
            Cmd::Alias("seekb", CmdKind::SeekBackward, "seek-back"),
            Cmd::Normal("volume", CmdKind::Volume),
            Cmd::Alias("vol", CmdKind::Volume, "volume"),
            Cmd::Normal("volume-up", CmdKind::VolumeUp),
            Cmd::Alias("volup", CmdKind::VolumeUp, "volume-up"),
            Cmd::Normal("volume-down", CmdKind::VolumeDown),
            Cmd::Alias("voldown", CmdKind::VolumeDown, "volume-down"),
            Cmd::Normal("volume-reset", CmdKind::VolumeReset),
            Cmd::Alias("volreset", CmdKind::VolumeReset, "volume-reset"),
            Cmd::Normal("mute", CmdKind::Mute),
            Cmd::Normal("unmute", CmdKind::Unmute),
            Cmd::Normal("mute-toggle", CmdKind::MuteToggle),
            Cmd::Alias("mutetog", CmdKind::MuteToggle, "mute-toggle"),
            Cmd::Normal("loop-none", CmdKind::LoopNone),
            Cmd::Normal("loop-queue", CmdKind::LoopQueue),
            Cmd::Normal("loop-shuffle", CmdKind::LoopShuffle),

            Cmd::Normal("queue-add", CmdKind::QueueAdd),
            Cmd::Alias("add", CmdKind::QueueAdd, "queue-add"),
            Cmd::Normal("queue-clear", CmdKind::QueueClear),
            Cmd::Alias("clear", CmdKind::QueueClear, "queue-clear"),
            Cmd::Normal("queue-shuffle", CmdKind::QueueShuffle),
            Cmd::Alias("shuffle", CmdKind::QueueShuffle, "queue-shuffle"),
        ] }
    }

    /// Returns formatted list of the commands:
    /// `(is_alias, "command <ARGS>", "(alias to :command) Command description")`
    pub fn formatted_list(&self) -> Vec<(bool, String, String)> {
        let mut result = vec![];

        for cmd in &self.list {
            let alias = match cmd {
                Cmd::Normal(_, _) => None,
                Cmd::Alias(_, _, to) => Some(to)
            };

            let name = cmd.name();
            let kind = cmd.kind();
            let args = kind.args();

            let name = match args {
                Some(args) => format!("{} {}", name, args),
                None => name.to_string()
            };
            let desc = kind.description();
            let desc = match alias {
                Some(alias) => format!("(alias to :{alias}) {desc}"),
                None => desc.to_string()
            };

            result.push((alias.is_some(), name, desc));
        }

        result
    }

    pub fn find<S: AsRef<str>>(&self, name: S) -> Option<&Cmd> {
        let index = self.list
            .iter()
            .position(|c| c.name().eq(name.as_ref()))?;
        Some(&self.list[index])
    }
}

/// Execute command with args by given string
/// For example: `"queue-add ~/my-cool-music-dir/*"`
pub fn exec_command<S: AsRef<str>>(ctx: &mut AppContext, command: S) -> Result<Action, UpdateError> {
    let command = command.as_ref().trim();
    let (cmd_name, args_str) = match command.split_once(' ') {
        Some((cmd, args)) => (cmd, args.trim()),
        None => (command, "")
    };
    let args: Vec<&str> = args_str
        .split(' ')
        .filter(|a| !a.is_empty())
        .collect();

    let first_arg = args.first();

    let cmd = ctx.commands.find(cmd_name)
        .ok_or(CmdError::NoSuchCmd)?;

    match cmd.kind() {
        CmdKind::Quit => return Ok(Action::Quit),
        CmdKind::Hello => ctx.state.notify("hey"),
        CmdKind::Echo => ctx.state.notify(args_str),

        CmdKind::PlayNext => ctx.player.play_next()?,
        CmdKind::PlayPrev => ctx.player.play_prev()?,
        CmdKind::Replay => ctx.player.replay()?,
        CmdKind::Resume => ctx.player.resume()?,
        CmdKind::Pause => ctx.player.pause()?,
        CmdKind::Stop => ctx.player.stop()?,
        CmdKind::Toggle => ctx.player.toggle()?,
        CmdKind::Seek => ctx.player.seek(parse_secs(args.first())?)?,
        CmdKind::SeekForward => ctx.player.seek_forward(parse_secs(first_arg)?)?,
        CmdKind::SeekBackward => ctx.player.seek_backward(parse_secs(first_arg)?)?,
        CmdKind::Volume => ctx.player.set_volume(parse_percent(first_arg)?)?,
        CmdKind::VolumeUp => ctx.player.volume_up(parse_percent(first_arg)?)?,
        CmdKind::VolumeDown => ctx.player.volume_down(parse_percent(first_arg)?)?,
        CmdKind::VolumeReset => ctx.player.set_volume(1.0)?,
        CmdKind::Mute => ctx.player.set_muted(true)?,
        CmdKind::Unmute => ctx.player.set_muted(false)?,
        CmdKind::MuteToggle => ctx.player.mute_toggle()?,
        CmdKind::LoopNone => ctx.player.set_loop(LoopState::None),
        CmdKind::LoopQueue => ctx.player.set_loop(LoopState::Queue),
        CmdKind::LoopShuffle => ctx.player.set_loop(LoopState::Shuffle),

        CmdKind::QueueAdd => cmd_add(ctx, args)?,
        CmdKind::QueueClear => ctx.player.queue_clear()?,
        CmdKind::QueueShuffle => ctx.player.queue_shuffle(),
    }

    Ok(Action::Draw)
}

fn cmd_add(ctx: &mut AppContext, args: Vec<&str>) -> Result<(), UpdateError> {
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
            let Ok(track) = Track::from_path(&mut ctx.cache, path) else {
                continue;
            };

            tracks.push(Rc::new(QueueTrack::Signle(Rc::new(track))));
        }
    }

    ctx.state.notify(format!("{} tracks were added", tracks.len()));
    ctx.player.queue_add_tracks(tracks);
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
