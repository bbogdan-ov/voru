use tuich::{
    buffer::Buffer,
    event::Key,
    layout::{Rect, Stack},
    widget::{Draw, RefDraw}
};

use crate::{
    app::{State, View},
    config::Config,
    match_keys,
    player::{PlaybackError, PlaybackResult, Player},
    traits::ToReadable,
    widget::{List, ListState, PlaylistWidget, TrackTable, TrackWidget, ViewWidget},
    Action,
};

/// Playlists view
#[derive(Debug)]
pub struct PlaylistsView {
    playlists_state: ListState,
    tracks_state: ListState,
}
impl PlaylistsView {
    pub fn new() -> Self {
        Self {
            playlists_state: ListState::new(),
            tracks_state: ListState::new(),
        }
    }

    fn play_playlist(&mut self, player: &mut Player) -> PlaybackResult {
        player.play_playlist(self.cur_playlist(), 0)
    }
    fn play_track(&mut self, player: &mut Player) -> PlaybackResult {
        player.play_playlist(self.cur_playlist(), self.cur_track())
    }

    pub fn handle_key(&mut self, config: &Config, player: &mut Player, state: &State, key: Key) -> Result<Action, PlaybackError> {
        match state.view {
            View::Playlists => self.handle_playlists_key(config, player, key),
            View::Tracks => self.handle_tracks_key(config, player, key),
            _ => Ok(Action::Nope)
        }
    }
    fn handle_playlists_key(&mut self, config: &Config, player: &mut Player, key: Key) -> Result<Action, PlaybackError> {
        match_keys! {
            config, key,

            play => self.play_playlist(player)?,
            play_shuffled => {
                self.play_playlist(player)?;
                player.queue_shuffle();
                player.play(0)?;
            }
            queue_add => player.queue_add_playlist(self.cur_playlist())?;
            else {
                return Ok(self.playlists_state.handle_key(config, key).into())
            }
        }

        Ok(Action::Draw)
    }
    fn handle_tracks_key(&mut self, config: &Config, player: &mut Player, key: Key) -> Result<Action, PlaybackError> {
        match_keys! {
            config, key,

            play => self.play_track(player)?,
            play_shuffled => {
                self.play_track(player)?;
                player.queue_shuffle();
                player.queue.swap(player.cur_track_index.unwrap(), 0);
                player.cur_track_index = Some(0);
            }
            queue_add => player.queue_add_from_playlist(self.cur_playlist(), self.cur_track())?;

            else {
                return Ok(self.tracks_state.handle_key(config, key).into());
            }
        }

        Ok(Action::Draw)
    }

    pub fn draw(&mut self, config: &Config, player: &Player, state: &State, buf: &mut Buffer, rect: Rect) -> Rect {
        let rects = Stack::row(&[1, 2])
            .gap(1)
            .calc(rect);

        let playstate = player.playstate();

        self.playlists_state.active = state.view == View::Playlists;
        self.tracks_state.active = state.view == View::Tracks;

        let playlists_rect = ViewWidget::new(config, playstate, "Playlists")
            .with_desc(player.playlists.len().to_string())
            .with_active(self.playlists_state.active)
            .draw(buf, rects[0]);

        // Draw playlists list
        List::new(&mut self.playlists_state, &player.playlists)
            .draw(buf, playlists_rect, |index, playlist, list_state, buf, rect| {
                let playlist = playlist.borrow();
                PlaylistWidget {
                    index,
                    config,
                    state: list_state,
                    playstate,
                    playlist: &playlist,
                    playing: player.is_playlist_index_current(&index)
                }.draw(buf, rect)
            });

        // Draw tracks list
        if let Some(playlist) = player.playlists.get(self.cur_playlist()) {
            let playlist = playlist.borrow();
            let tracks_count = playlist.tracks.len();
            let desc = format!("{} tracks  {}", tracks_count, playlist.duration.to_readable());

            let tracks_rect = ViewWidget::new(config, playstate, format!("{}", playlist.name))
                .with_desc(desc)
                .with_active(self.tracks_state.active)
                .draw(buf, rects[1]);

            let table = TrackTable::new(tracks_count, tracks_rect);
            
            List::new(&mut self.tracks_state, &playlist.tracks)
                .draw(buf, tracks_rect, |index, track, list_state, buf, rect| {
                    TrackWidget {
                        index,
                        config,
                        state: list_state,
                        playstate,
                        track,
                        playing: player.is_track_current(&track.id)
                    }.draw(&table, buf, rect)
                });
        } else {
            ViewWidget::new(config, playstate, "Tracks")
                .with_active(self.tracks_state.active)
                .draw(buf, rects[1]);
        }

        rect
    }

    // Get

    fn cur_playlist(&self) -> usize {
        self.playlists_state.current()
    }
    fn cur_track(&self) -> usize {
        self.tracks_state.current()
    }
}
