use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
    fs,
    io::{self, Read, Seek},
    ops::Deref,
    path::Path,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use thiserror::Error;

use crate::{
    playlist::Playlist,
    track::{Id, Track},
    traits::{MoveTo, Shuffle},
};

// Consts
pub const MAX_VOLUME: f32 = 2.0;

// Errors
#[derive(Debug, Error)]
pub enum PlaybackError {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("[rodio] Play error: {0}")]
    Play(rodio::PlayError),
    #[error("[rodio] Seek error: {0}")]
    Seek(rodio::source::SeekError),
    #[error("No audio is currently playing")]
    NoAudio,
    #[error("No such track")]
    NoTrack,
    #[error("No such playlist")]
    NoPlaylist,
    #[error("Nothing is being playing")]
    NotPlaying,
    #[error("No more tracks to play")]
    NoMore,
    #[error("Queue is empty")]
    EmptyQueue
}
pub type PlaybackResult = Result<(), PlaybackError>;

/// Play state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Playing,
    Paused,
    Stopped,
    Ended
}
impl Display for PlayState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Playing => write!(f, "playing"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
            Self::Ended => write!(f, "ended"),
        }
    }
}

/// Playback
pub struct Playback {
    stream_handle: OutputStreamHandle,

    sink: Option<Arc<Sink>>,
    duration: Option<Duration>
}
impl Playback {
    fn play_path<P: AsRef<Path>>(&mut self, path: P, duration: Option<Duration>) -> PlaybackResult {
        let file = fs::File::open(path)
            .map_err(PlaybackError::Io)?;
        self.play_file(file, duration)
    }
    fn play_file<F: Read + Seek + Send + Sync + 'static>(
        &mut self,
        file: F,
        duration: Option<Duration>
    ) -> PlaybackResult {
        if let Some(sink) = &self.sink {
            sink.stop();
        }

        let sink = Arc::new(Sink::try_new(&self.stream_handle)
            .map_err(PlaybackError::Play)?);
        let source = Decoder::new(file)
            .map_err(|e| PlaybackError::Play(e.into()))?;
        let clonned_sink = Arc::clone(&sink);

        self.duration = duration.or(source.total_duration());

        sink.append(source);

        self.sink = Some(sink);

        std::thread::spawn(move || {
            clonned_sink.sleep_until_end()
        });

        Ok(())
    }

    fn resume(&mut self) -> PlaybackResult {
        let sink = self.sink
            .as_ref()
            .ok_or(PlaybackError::NoAudio)?;

        sink.play();
        Ok(())
    }
    fn pause(&mut self) -> PlaybackResult {
        let sink = self.sink
            .as_ref()
            .ok_or(PlaybackError::NoAudio)?;

        sink.pause();
        Ok(())
    }
    fn stop(&mut self) -> PlaybackResult {
        let sink = self.sink
            .as_ref()
            .ok_or(PlaybackError::NoAudio)?;

        sink.stop();
        self.sink = None;
        Ok(())
    }
    fn seek(&mut self, pos: Duration) -> PlaybackResult {
        let sink = self.sink
            .as_ref()
            .ok_or(PlaybackError::NoAudio)?;

        // Clamp seek pos, because sometimes rodio may drop an error
        let pos =
            if let Some(dur) = self.duration { pos.min(dur.saturating_sub(Duration::from_secs(1))) }
            else { pos };

        sink.try_seek(pos)
            .map_err(PlaybackError::Seek)
    }
    fn set_volume(&mut self, volume: f32) -> PlaybackResult {
        let sink = self.sink
            .as_ref()
            .ok_or(PlaybackError::NoAudio)?;

        sink.set_volume(volume);
        Ok(())
    }

    fn pos(&self) -> Option<Duration> {
        self.sink.as_ref().map(|s| s.get_pos())
    }
}

/// Queue track
#[derive(Debug)]
pub enum QueueTrack {
    Signle(Rc<Track>),
    Playlist(Rc<Track>, usize)
}
impl Deref for QueueTrack {
    type Target = Rc<Track>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Signle(track) => track,
            Self::Playlist(track, _) => track,
        }
    }
}

/// Player
pub struct Player {
    playback: Playback,

    pub queue: Vec<Rc<QueueTrack>>,
    pub playlists: Vec<Rc<RefCell<Playlist>>>,
    pub queue_dur: Duration,
    pub elapsed: Duration,

    pub cur_track_index: Option<usize>,
    pub cur_track: Option<Rc<QueueTrack>>,

    volume: f32,
    muted: bool
}
impl Player {
    pub fn new(stream_handle: OutputStreamHandle, mut playlists: Vec<Rc<RefCell<Playlist>>>) -> Self {
        // Collect all the tracks from the playlists and put them into the * playlist
        let mut all_tracks = vec![];
        for playlist in &playlists {
            // Clonning the vector of the Rc's
            all_tracks.extend(playlist.borrow().tracks.clone());
        }
        playlists.insert(0, Rc::new(RefCell::new(Playlist::new("*", all_tracks))));

        Self {
            playback: Playback {
                stream_handle,
                sink: None,
                duration: None
            },

            queue: vec![],
            playlists,
            queue_dur: Duration::default(),
            elapsed: Duration::default(),

            cur_track_index: None,
            cur_track: None,

            volume: 1.0,
            muted: false
        }
    }

    pub fn handle_tick(&mut self) {
        if self.cur_track.is_some() {
            let playstate = self.playstate();

            // Play a next track if the current one is ended
            if !self.current_is_last() && playstate == PlayState::Ended {
                let _ = self.play_next();
            }
        }
    }

    fn update_queue_dur(&mut self) {
        self.queue_dur = self.queue
            .iter()
            .fold(Duration::default(), |acc, t| acc + t.duration());
    }
    fn update_elapsed(&mut self) {
        if let Some(cur_index) = self.cur_track_index {
            self.elapsed = self.queue[..cur_index]
                .iter()
                .fold(Duration::default(), |acc, t| acc + t.duration());
        } else {
            self.elapsed = Duration::default();
        }
    }

    /// Play a track from the queue
    pub fn play(&mut self, track_index: usize) -> PlaybackResult {
        let Some(track) = self.queue.get(track_index) else {
            return Err(PlaybackError::NoTrack);
        };

        self.playback.play_path(&track.path, track.try_duration().cloned())?;

        if self.muted {
            self.playback.set_volume(0.0)?;
        } else {
            self.playback.set_volume(self.volume)?;
        }

        self.cur_track_index = Some(track_index);
        self.cur_track = Some(Rc::clone(track));

        self.update_elapsed();
        Ok(())
    }
    pub fn play_playlist(&mut self, playlist_index: usize, track_index: usize) -> PlaybackResult {
        self.queue_set_playlist(playlist_index)?;
        self.play(track_index)
    }
    /// Play the first track in the queue
    pub fn replay(&mut self) -> PlaybackResult {
        self.play(0)
    }
    pub fn play_next(&mut self) -> PlaybackResult {
        let index = self.cur_track_index
            .ok_or(PlaybackError::NotPlaying)?;
        if index >= self.queue.len().saturating_sub(1) {
            return Err(PlaybackError::NoMore);
        }

        self.play(index + 1)
    }
    pub fn play_prev(&mut self) -> PlaybackResult {
        let index = self.cur_track_index
            .ok_or(PlaybackError::NotPlaying)?;
        if index == 0 {
            return Err(PlaybackError::NoMore);
        }

        self.play(index - 1)
    }
    pub fn resume(&mut self) -> PlaybackResult {
        self.playback.resume()
    }
    pub fn pause(&mut self) -> PlaybackResult {
        self.playback.pause()
    }
    pub fn stop(&mut self) -> PlaybackResult {
        self.cur_track = None;
        self.cur_track_index = None;
        self.playback.stop()
    }
    pub fn toggle(&mut self) -> PlaybackResult {
        match self.playstate() {
            PlayState::Paused |
            PlayState::Ended |
            PlayState::Stopped => self.resume(),
            PlayState::Playing => self.pause()
        }
    }
    pub fn seek(&mut self, pos: Duration) -> PlaybackResult {
        self.playback.seek(pos)
    }
    pub fn seek_forward(&mut self, dur: Duration) -> PlaybackResult {
        self.seek(self.pos() + dur)
    }
    pub fn seek_backward(&mut self, dur: Duration) -> PlaybackResult {
        self.seek(self.pos().saturating_sub(dur))
    }
    pub fn set_volume(&mut self, volume: f32) -> PlaybackResult  {
        self.volume = volume.clamp(0.0, MAX_VOLUME);

        if self.muted {
            self.playback.set_volume(0.0)
        } else {
            self.playback.set_volume(self.volume)
        }
    }
    pub fn volume_up(&mut self, value: f32) -> PlaybackResult {
        self.set_volume(self.volume + value)
    }
    pub fn volume_down(&mut self, value: f32) -> PlaybackResult {
        self.set_volume(self.volume - value)
    }
    pub fn set_muted(&mut self, muted: bool) -> PlaybackResult {
        self.muted = muted;

        if self.muted {
            self.playback.set_volume(0.0)
        } else {
            self.playback.set_volume(self.volume)
        }
    }
    pub fn mute_toggle(&mut self) -> PlaybackResult {
        self.set_muted(!self.muted)
    }

    /// Returns the current track position
    /// If nothing is playing, returns zero duration
    pub fn pos(&self) -> Duration {
        self.playback.pos().unwrap_or_default()
    }
    /// Returns the current track duration
    /// If nothing is playing, returns zero duration
    pub fn duration(&self) -> Duration {
        self.cur_track
            .as_ref()
            .and_then(|t| t.try_duration().cloned())
            .unwrap_or_default()
    }
    pub fn volume(&self) -> f32 {
        self.volume
    }
    pub fn muted(&self) -> bool {
        self.muted
    }
    pub fn playstate(&self) -> PlayState {
        if self.playback.sink.is_none() {
            PlayState::Stopped
        } else if self.playback.sink.as_ref().is_some_and(|s| s.empty()) {
            PlayState::Ended
        } else if self.playback.sink.as_ref().is_some_and(|s| s.is_paused()) {
            PlayState::Paused
        } else {
            PlayState::Playing
        }
    }
    pub fn is_track_current(&self, track_id: &Id) -> bool {
        self.cur_track
            .as_ref()
            .is_some_and(|t| t.id.eq(track_id))
    }
    pub fn is_track_index_current(&self, track_index: &usize) -> bool {
        self.cur_track_index
            .as_ref()
            .is_some_and(|i| i.eq(track_index))
    }
    pub fn is_playlist_index_current(&self, playlist_index: &usize) -> bool {
        match self.cur_track.as_deref() {
            Some(QueueTrack::Playlist(_, index)) => index.eq(playlist_index),
            _ => false
        }
    }
    /// Returns whether current track is last in the queue
    pub fn current_is_last(&self) -> bool {
        self.cur_track_index.is_some_and(|i| i >= self.queue.len().saturating_sub(1))
    }

    // Playlists
    /// Returns a reference to a playlist by its index
    pub fn playlist_get(&self, index: usize) -> Option<Ref<'_, Playlist>> {
        self.playlists.get(index)
            .map(|p| RefCell::borrow(p))
    }
    /// Returns a mutable reference to a playlist by its index
    pub fn playlist_get_mut(&mut self, index: usize) -> Option<RefMut<'_, Playlist>> {
        self.playlists.get_mut(index)
            .map(|p| RefCell::borrow_mut(p))
    }

    // Queue
    /// Add a track to the end of the queue
    pub fn queue_add(&mut self, track: Rc<QueueTrack>) {
        self.queue.push(track);
        self.update_queue_dur();
    }
    /// Add tracks to the end of the queue
    pub fn queue_add_tracks(&mut self, tracks: Vec<Rc<QueueTrack>>) {
        self.queue.extend(tracks);
        self.update_queue_dur();
    }
    /// Add playlist to the end of the queue
    pub fn queue_add_playlist(&mut self, playlist_index: usize) -> PlaybackResult {
        let playlist = self.playlists
            .get(playlist_index)
            .ok_or(PlaybackError::NoPlaylist)?;

        let mut tracks = vec![];
        for track in &playlist.borrow().tracks {
            tracks.push(Rc::new(QueueTrack::Playlist(Rc::clone(track), playlist_index)))
        }

        self.queue_add_tracks(tracks);
        Ok(())
    }
    pub fn queue_add_from_playlist(&mut self, playlist_index: usize, track_index: usize) -> PlaybackResult {
        let playlist = self.playlists
            .get(playlist_index)
            .ok_or(PlaybackError::NoPlaylist)?
            .borrow();
        let track = playlist.tracks
            .get(track_index)
            .ok_or(PlaybackError::NoTrack)?;
        let track = Rc::clone(track);

        drop(playlist);

        self.queue_add(QueueTrack::Playlist(track, playlist_index).into());
        Ok(())
    }
    /// Clear and add tracks to the queue
    pub fn queue_set(&mut self, tracks: Vec<Rc<QueueTrack>>) -> PlaybackResult {
        self.queue = tracks;
        self.update_queue_dur();
        self.stop()
    }
    pub fn queue_set_playlist(&mut self, playlist_index: usize) -> PlaybackResult {
        self.queue.clear();
        self.queue_add_playlist(playlist_index)
    }
    /// Clear queue
    pub fn queue_clear(&mut self) -> PlaybackResult {
        self.queue.clear();
        self.update_queue_dur();
        self.stop()
    }
    /// Randomize the queue order
    pub fn queue_shuffle(&mut self) {
        self.queue.shuffle();

        if let Some(cur_track) = &self.cur_track {
            if let Some(new_index) = self.queue.iter().position(|t| t.id == cur_track.id) {
                self.cur_track_index = Some(new_index);
                self.update_elapsed();
            }
        }
    }
    /// Remove a track from the queue
    pub fn queue_remove(&mut self, index: usize) -> PlaybackResult {
        if self.queue.is_empty() {
            return Err(PlaybackError::EmptyQueue)
        }

        if self.is_track_index_current(&index) {
            self.queue.remove(index);

            if self.queue.is_empty() {
                self.stop()?;
            } else {
                self.play(index)?;
            }
        } else {
            self.queue.remove(index);

            if let Some(cur_index) = self.cur_track_index {
                if cur_index > index {
                    self.cur_track_index = Some(cur_index.saturating_sub(1));
                }
            }
        }

        self.update_queue_dur();
        self.update_elapsed();
        Ok(())
    }
    /// Move a track in the queue to some position
    pub fn queue_move_to(&mut self, track_index: usize, to_index: usize) -> PlaybackResult {
        let queue_len = self.queue.len();
        if track_index >= queue_len {
            return Err(PlaybackError::NoTrack);
        }
        let to_index = to_index.min(queue_len.saturating_sub(1));
        
        self.queue.move_to(track_index, to_index);

        if self.is_track_index_current(&to_index) {
            self.cur_track_index = Some(track_index);
        } else if self.is_track_index_current(&track_index) {
            self.cur_track_index = Some(to_index);
        }

        self.update_elapsed();
        Ok(())
    }
}
