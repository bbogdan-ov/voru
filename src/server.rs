use std::sync::{mpsc, Arc, Mutex};

use mpris_server as mpris;
use mpris::zbus::{self, fdo};

use crate::{player::PlayerState, UpdateKind};

/// Server action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerAction {
    Play,
    Pause,
    Stop,
    PlayPause,
    Seek(mpris::Time),
    Volume(f32),

    Next,
    Prev,
    Shuffle
}

/// Server
pub struct Server {
    pub state: Arc<Mutex<PlayerState>>,
    pub sender: mpsc::Sender<UpdateKind>
}
impl Server {
    fn send(&self, action: ServerAction) -> fdo::Result<()> {
        self.sender.send(UpdateKind::Server(action)).unwrap();
        Ok(())
    }
}
impl mpris::RootInterface for Server {
    async fn quit(&self) -> fdo::Result<()> {
        Ok(())
    }
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }
    async fn set_fullscreen(&self, _fullscreen: bool) -> zbus::Result<()> {
        Ok(())
    }
    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn identity(&self) -> fdo::Result<String> {
        Ok("VORU".into())
    }
    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("".into())
    }
    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec!["audio/mpeg".into()])
    }
    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec!["file".into()])
    }
}
impl mpris::PlayerInterface for Server {
    async fn play(&self) -> fdo::Result<()> {
        self.send(ServerAction::Play)
    }
    async fn pause(&self) -> fdo::Result<()> {
        self.send(ServerAction::Pause)
    }
    async fn stop(&self) -> fdo::Result<()> {
        self.send(ServerAction::Stop)
    }
    async fn play_pause(&self) -> fdo::Result<()> {
        self.send(ServerAction::PlayPause)
    }
    async fn seek(&self, offset: mpris::Time) -> fdo::Result<()> {
        self.send(ServerAction::Seek(offset))
    }

    async fn next(&self) -> fdo::Result<()> {
        self.send(ServerAction::Next)
    }
    async fn previous(&self) -> fdo::Result<()> {
        self.send(ServerAction::Prev)
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> { Ok(()) }

    async fn playback_status(&self) -> fdo::Result<mpris::PlaybackStatus> {
        Ok(self.state.lock().unwrap()
            .status)
    }
    async fn metadata(&self) -> fdo::Result<mpris::Metadata> {
        Ok(self.state.lock().unwrap()
            .metadata.clone())
    }
    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_control(&self) -> fdo::Result<bool> { Ok(true) }
    async fn minimum_rate(&self) -> fdo::Result<mpris::PlaybackRate> { Ok(1.0) }
    async fn maximum_rate(&self) -> fdo::Result<mpris::PlaybackRate> { Ok(1.0) }

    async fn set_loop_status(&self, _loop_status: mpris::LoopStatus) -> zbus::Result<()> {
        // TODO:
        Ok(())
    }
    async fn loop_status(&self) -> fdo::Result<mpris::LoopStatus> {
        // TODO:
        Ok(mpris::LoopStatus::None)
    }
    async fn set_rate(&self, _rate: mpris::PlaybackRate) -> zbus::Result<()> {
        Ok(())
    }
    async fn rate(&self) -> fdo::Result<mpris::PlaybackRate> {
        Ok(1.0)
    }
    async fn set_shuffle(&self, shuffle: bool) -> zbus::Result<()> {
        if shuffle {
            self.send(ServerAction::Shuffle)?;
        }
        Ok(())
    }
    async fn shuffle(&self) -> fdo::Result<bool> {
        // TODO:
        Ok(false)
    }
    async fn set_volume(&self, volume: mpris::Volume) -> zbus::Result<()> {
        self.send(ServerAction::Volume(volume as f32))?;
        Ok(())
    }
    async fn volume(&self) -> fdo::Result<mpris::Volume> {
        Ok(self.state.lock().unwrap().volume as f64)
    }
    async fn set_position(&self, _track_id: mpris::TrackId, _position: mpris_server::Time) -> fdo::Result<()> {
        // TODO:
        Ok(())
    }
    async fn position(&self) -> fdo::Result<mpris::Time> {
        Ok(self.state.lock().unwrap().pos)
    }
}
