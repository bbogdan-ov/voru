use tuich::{buffer::Buffer, layout::{Clip, Rect}, text::Text, widget::{Clear, Draw, RefDraw}};

use crate::{config::Config, player::PlayState, playlist::Playlist};

use super::ListState;

/// Playlist widget
pub struct PlaylistWidget<'a> {
    pub index: usize,
    pub config: &'a Config,
    pub state: &'a ListState,
    pub playstate: PlayState,
    pub playlist: &'a Playlist,
    pub playing: bool
}
impl<'a> RefDraw for PlaylistWidget<'a> {
    fn draw(&self, buf: &mut Buffer, rect: Rect) -> Rect {
        let rect = rect.with_height(1);

        let is_cur = self.state.active && self.index == self.state.current();
        let is_paused = self.playing && self.playstate != PlayState::Playing;

        let style =
            if is_cur && is_paused { self.config.theme.playlist.selected_paused }
            else if is_paused { self.config.theme.playlist.paused }

            else if is_cur && self.playing { self.config.theme.playlist.selected_playing }
            else if self.playing { self.config.theme.playlist.playing }

            else if is_cur { self.config.theme.playlist.selected }
            else { self.config.theme.playlist.normal };

        Clear::new(style)
            .draw(buf, rect);

        Text::new(&self.playlist.name, ())
            .clip(Clip::Ellipsis)
            .draw(buf, rect.margin((1, 0)));

        rect
    }
}
