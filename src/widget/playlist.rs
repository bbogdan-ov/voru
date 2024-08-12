use tuich::{buffer::Buffer, layout::{Clip, Rect}, text::Text, widget::{Clear, Draw, RefDraw}};

use crate::{app::AppContext, player::PlayState, playlist::Playlist};

use super::ListState;

/// Playlist widget
pub struct PlaylistWidget<'a> {
    pub index: usize,
    pub state: &'a ListState,
    pub ctx: &'a AppContext,
    pub playlist: &'a Playlist,
    pub playing: bool
}
impl<'a> RefDraw for PlaylistWidget<'a> {
    fn draw(&self, buf: &mut Buffer, rect: Rect) -> Rect {
        let theme = &self.ctx.config.theme;

        let rect = rect.with_height(1);

        let is_cur = self.state.active && self.index == self.state.current();
        let is_paused = self.playing && self.ctx.player.playstate() != PlayState::Playing;

        let style =
            if is_cur && is_paused { theme.playlist_selected_paused }
            else if is_paused { theme.playlist_paused }

            else if is_cur && self.playing { theme.playlist_selected_playing }
            else if self.playing { theme.playlist_playing }

            else if is_cur { theme.playlist_selected }
            else { theme.playlist };

        Clear::new(style)
            .draw(buf, rect);

        Text::new(&self.playlist.name, ())
            .clip(Clip::Ellipsis)
            .draw(buf, rect.margin((1, 0)));

        rect
    }
}
