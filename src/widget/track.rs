use tuich::{buffer::{Buffer, Cell}, layout::{Align, Clip, Rect, Stack}, style::Color, text::Text, widget::{Clear, Draw}};

use crate::{app::AppContext, player::PlayState, track::Track, traits::ToReadable};

use super::ListState;

/// Track table
#[derive(Debug, Clone)]
pub struct TrackTable {
    index_rect: Rect,
    title_rect: Rect,
    artist_rect: Rect
}
impl TrackTable {
    pub fn new(tracks_count: usize, rect: Rect) -> Self {
        let index_width =
            if tracks_count <= 9 { 3 }
            else if tracks_count <= 99 { 4 }
            else if tracks_count <= 999 { 5 }
            else { 6 };

        let stack_rect = rect
            .with_height(1)
            .margin_left(index_width + 1)
            .margin_right(1);
        let rects = Stack::row(&[2, 1])
            .gap(2)
            .calc(stack_rect);

        Self {
            index_rect: rect.with_width(index_width),
            title_rect: rects[0],
            artist_rect: rects[1],
        }
    }
}

/// Track widget
pub struct TrackWidget<'a> {
    pub index: usize,
    pub state: &'a ListState,
    pub ctx: &'a AppContext,
    pub track: &'a Track,
    pub playing: bool,
}
impl<'a> TrackWidget<'a> {
    pub fn draw(&self, table: &TrackTable, buf: &mut Buffer, rect: Rect) -> Rect {
        let theme = &self.ctx.config.theme;

        let rect = rect.with_height(1);
        let index_rect = table.index_rect.with_y(rect.y);
        let title_rect = table.title_rect.with_y(rect.y);
        let artist_rect = table.artist_rect.with_y(rect.y);

        let is_cur = self.state.active && self.state.current() == self.index;
        let is_paused = self.playing && self.ctx.player.playstate() != PlayState::Playing;

        let style =
            if is_cur && is_paused { theme.track_selected_paused }
            else if is_paused { theme.track_paused }

            else if is_cur && self.playing { theme.track_selected_playing }
            else if self.playing { theme.track_playing }

            else if is_cur { theme.track_selected }
            else { theme.track };

        let title = self.track.title();

        // Draw index
        Text::from(format!("{}.", self.index + 1))
            .align(Align::End)
            .draw(buf, index_rect);
        // Draw title
        let title_text_rect = Text::from(title)
            .clip(Clip::Ellipsis)
            .draw(buf, title_rect);
        // Draw album
        if let Some(album) = self.track.try_album() {
            Text::from(format!("- {}", album))
                .style(Color::Gray)
                .clip(Clip::Ellipsis)
                .draw(buf, title_rect.margin_left(title_text_rect.width + 1));
        }
        // Draw duration
        let dur_width = if let Some(dur) = self.track.try_duration() {
            Text::from(dur.to_readable())
                .align(Align::End)
                .draw(buf, artist_rect)
                .width + 2
        } else { 0 };
        // Draw artist
        if let Some(artist) = self.track.try_artist() {
            Text::from(artist)
                .clip(Clip::Ellipsis)
                .draw(buf, artist_rect.margin_right(dur_width));
        }

        // Fill the item with some color
        Clear::new(Cell::empty(style))
            .draw(buf, rect);

        rect
    }
}
