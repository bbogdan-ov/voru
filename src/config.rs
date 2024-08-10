use std::{env::var, fs, io, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tuich::{event::{Key, KeyCode, KeyMod}, style::{Color, Style, Stylized}};

use crate::{key, widget::PlayerStyle};

// Errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("Bad syntax: {0}")]
    Parse(toml::de::Error),
    #[error("$HOME variable not found")]
    NoHomeVar
}

// Sections
/// Config theme item
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigThemeItem {
    pub normal: Style,
    pub selected: Style,
    pub playing: Style,
    pub selected_playing: Style,
    pub paused: Style,
    pub selected_paused: Style,
}
impl Default for ConfigThemeItem {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            selected: Style::cleared().reverse(true),
            playing: Color::Green.into(),
            selected_playing: Color::Green.reverse(),
            paused: Color::Blue.into(),
            selected_paused: Color::Blue.reverse(),
        }
    }
}
/// Config theme title
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigThemeTitle {
    pub active_playing: Style,
    pub active_paused: Style,
    pub inactive: Style
}
impl Default for ConfigThemeTitle {
    fn default() -> Self {
        Self {
            active_playing: Color::Green.into(),
            active_paused: Color::Blue.into(),
            inactive: Color::Gray.into()
        }
    }
}
/// Config theme player
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigThemePlayer {
    pub playing: Style,
    pub paused: Style,
    pub stopped: Style
}
impl Default for ConfigThemePlayer {
    fn default() -> Self {
        Self {
            playing: Color::Green.into(),
            paused: Color::Blue.into(),
            stopped: Color::Gray.into(),
        }
    }
}
/// Config theme
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigTheme {
    pub title: ConfigThemeTitle,
    pub player: ConfigThemePlayer,

    pub playlist: ConfigThemeItem,
    pub track: ConfigThemeItem,

    pub notif_normal: Style,
    pub notif_error: Style,
    pub cmdline: Style,
    pub completion: Style,
    pub completion_alias: Style,
}
impl Default for ConfigTheme {
    fn default() -> Self {
        Self {
            title: ConfigThemeTitle::default(),
            player: ConfigThemePlayer::default(),

            playlist: ConfigThemeItem::default(),
            track: ConfigThemeItem::default(),

            notif_normal: Style::cleared().fg(Color::Black).bg(Color::Blue),
            notif_error: Style::cleared().fg(Color::Black).bg(Color::Red),
            cmdline: Style::cleared().fg(Color::Black).bg(Color::Magenta),
            completion: Style::cleared().fg(Color::Black).bg(Color::Magenta),
            completion_alias: Style::cleared().fg(Color::Black).bg(Color::Magenta).italic(true),
        }
    }
}

/// Config format
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFormat {
    pub progress: char,
    pub progress_track: char,
    pub progress_thumb: String,
}
impl Default for ConfigFormat {
    fn default() -> Self {
        Self {
            progress: '─',
            progress_track: '─',
            progress_thumb: "".into(),
        }
    }
}

/// Config style
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigStyle {
    pub player: PlayerStyle,
}

/// Config layout
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigLayout {
    pub max_width: u16,
    pub max_height: u16,
    pub padding_x: u16,
    pub padding_y: u16,
    pub player_max_width: u16
}
impl Default for ConfigLayout {
    fn default() -> Self {
        Self {
            max_width: 90,
            max_height: 22,
            padding_x: 2,
            padding_y: 1,
            player_max_width: 80
        }
    }
}

type Keymap = Vec<Key>;

/// Config keys
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigKeys {
    pub quit: Option<Keymap>,
    pub escape: Option<Keymap>,
    pub enter: Option<Keymap>,
    pub complete: Option<Keymap>,
    pub next_history: Option<Keymap>,
    pub prev_history: Option<Keymap>,

    pub enter_cmd: Option<Keymap>,

    pub choose_item: Option<Keymap>,
    pub select_next_item: Option<Keymap>,
    pub select_prev_item: Option<Keymap>,
    pub select_next_item_fast: Option<Keymap>,
    pub select_prev_item_fast: Option<Keymap>,
    pub select_next_item_super_fast: Option<Keymap>,
    pub select_prev_item_super_fast: Option<Keymap>,
    pub select_first_item: Option<Keymap>,
    pub select_last_item: Option<Keymap>,

    pub next_view: Option<Keymap>,
    pub prev_view: Option<Keymap>,

    /// Focus on currently playing track in the queue
    pub queue_focus: Option<Keymap>,
    /// Move a selected track up the queue
    pub queue_move_up: Option<Keymap>,
    /// Move a selected track down the queue
    pub queue_move_down: Option<Keymap>,
    /// Remove a track from the queue
    pub queue_remove: Option<Keymap>,
    /// Add a track or playlist to the end of the queue
    pub queue_add: Option<Keymap>,
    pub queue_shuffle: Option<Keymap>,

    pub play: Option<Keymap>,
    pub play_shuffled: Option<Keymap>,
    pub play_next: Option<Keymap>,
    pub play_prev: Option<Keymap>,
    pub replay: Option<Keymap>,
    pub resume: Option<Keymap>,
    pub pause: Option<Keymap>,
    pub stop: Option<Keymap>,
    pub toggle: Option<Keymap>,
    pub seek_forward: Option<Keymap>,
    pub seek_backward: Option<Keymap>,
    pub seek_to_start: Option<Keymap>,
    pub volume_up: Option<Keymap>,
    pub volume_down: Option<Keymap>,
    pub volume_reset: Option<Keymap>,
    pub mute: Option<Keymap>,
    pub unmute: Option<Keymap>,
    pub mute_toggle: Option<Keymap>,
}
impl Default for ConfigKeys {
    fn default() -> Self {
        Self {
            quit: vec![ key!('Q') ].into(),
            escape: vec![ key!(Esc), key!(Ctrl + 'c'), key!(Ctrl + 'o') ].into(),
            enter: vec![ key!(Enter), key!(Ctrl + 'j') ].into(),
            complete: vec![ key!(Tab), key!(Ctrl + 'n'), key!(Ctrl + 'p') ].into(),
            next_history: vec![ key!(Down) ].into(),
            prev_history: vec![ key!(Up) ].into(),

            enter_cmd: vec![ key!(':'), key!(';') ].into(),

            choose_item: vec![ key!(Enter) ].into(),
            select_next_item: vec![ key!(Down), key!('j'), key!(Ctrl + 'n') ].into(),
            select_prev_item: vec![ key!(Up), key!('k'), key!(Ctrl + 'p') ].into(),
            select_next_item_fast: vec![ key!(Ctrl + 'd') ].into(),
            select_prev_item_fast: vec![ key!(Ctrl + 'u') ].into(),
            select_next_item_super_fast: vec![ key!(Ctrl + 'f'), key!(PageUp) ].into(),
            select_prev_item_super_fast: vec![ key!(Ctrl + 'b'), key!(PageDown) ].into(),
            select_first_item: vec![ key!('g'), key!(Home) ].into(),
            select_last_item: vec![ key!('G'), key!(End) ].into(),

            next_view: vec![ key!(Tab) ].into(),
            prev_view: vec![ key!(BackTab) ].into(),

            queue_focus: vec![ key!('f') ].into(),
            queue_move_up: vec![ key!(Shift + Up), key!('K') ].into(),
            queue_move_down: vec![ key!(Shift + Down), key!('J') ].into(),
            queue_remove: vec![ key!('D') ].into(),
            queue_add: vec![ key!('a') ].into(),
            queue_shuffle: vec![ key!('S') ].into(),

            play: vec![ key!(Enter) ].into(),
            play_shuffled: vec![ key!('P') ].into(),
            play_next: vec![ key!(Shift + Right), key!('L') ].into(),
            play_prev: vec![ key!(Shift + Left), key!('H') ].into(),
            replay: vec![ key!('y') ].into(),
            resume: None,
            pause: None,
            stop: None,
            toggle: vec![ key!(' ') ].into(),
            seek_forward: vec![ key!(Right), key!('l') ].into(),
            seek_backward: vec![ key!(Left), key!('h') ].into(),
            seek_to_start: None,
            volume_up: vec![ key!('+') ].into(),
            volume_down: vec![ key!('-') ].into(),
            volume_reset: vec![ key!('=') ].into(),
            mute: None,
            unmute: None,
            mute_toggle: vec![ key!('m') ].into(),
        }
    }
}

/// Config
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub playlists: Vec<PathBuf>,
    pub seek_jump: u64,
    pub volume_jump: f32,
    pub fast_jump: usize,
    pub super_fast_jump: usize,

    pub theme: ConfigTheme,
    pub style: ConfigStyle,
    pub format: ConfigFormat,
    pub layout: ConfigLayout,
    pub keys: ConfigKeys
}
impl Config {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        // Read the config file contents
        let content = fs::read_to_string(path)
            .map_err(ConfigError::Io)?;
        // Trying to parse a config from the file
        toml::from_str(&content)
            .map_err(ConfigError::Parse)
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            playlists: vec![],
            seek_jump: 10,
            volume_jump: 0.1,
            fast_jump: 10,
            super_fast_jump: 20,

            theme: ConfigTheme::default(),
            style: ConfigStyle::default(),
            format: ConfigFormat::default(),
            layout: ConfigLayout::default(),
            keys: ConfigKeys::default()
        }
    }
}

// Utils
pub fn default_config_path() -> Result<PathBuf, ConfigError> {
    let home = var("HOME")
        .map_err(|_| ConfigError::NoHomeVar)?;
    Ok(PathBuf::from(home).join(".config/voru/config.toml"))
}
