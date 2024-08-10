use serde::{Deserialize, Serialize};
use tuich::{buffer::Buffer, layout::{Align, Clip, Rect}, style::{Style, Stylized}, text::Text, widget::{Draw, RefDraw}};

use crate::{config::Config, player::{PlayState, Player}, traits::ToReadable};

use super::Progress;

/// Player style
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum PlayerStyle {
    #[default]
    Classic,
    ClassicReverse,
    Progress,
    Text
}

/// Player widget
pub struct PlayerWidget<'a> {
    pub config: &'a Config,
    pub player: &'a Player,
    pub style: PlayerStyle
}
impl<'a> PlayerWidget<'a> {
    pub fn style_rect(rect: Rect, style: PlayerStyle) -> Rect {
        match style {
            PlayerStyle::Classic |
            PlayerStyle::ClassicReverse => rect
                .with_height(2)
                .margin((1, 0)),

            PlayerStyle::Progress |
            PlayerStyle::Text => rect
                .with_height(1)
                .margin((1, 0)),
        }
    }
}
impl<'a> RefDraw for PlayerWidget<'a> {
    fn draw(&self, buf: &mut Buffer, rect: Rect) -> Rect {
        let rect = Self::style_rect(rect, self.style);

        let playstate = self.player.playstate();
        let style = match playstate {
            PlayState::Stopped => self.config.theme.player.stopped,
            PlayState::Playing => self.config.theme.player.playing,
            _ => self.config.theme.player.paused
        };

        match self.style {
            PlayerStyle::Classic |
            PlayerStyle::ClassicReverse => draw_classic(&self, style, buf, rect),

            PlayerStyle::Progress => draw_progress(&self, style, buf, rect),

            PlayerStyle::Text => draw_info(&self, style, buf, rect),
        }
    }
}

// Draw styles
fn draw_info(widget: &PlayerWidget, style: Style, buf: &mut Buffer, rect: Rect) -> Rect {
    let rect = rect.with_height(1);

    if let Some(track) = &widget.player.cur_track {
        let playstate = widget.player.playstate();
        let title = track.title();
        let volume =
            if widget.player.muted() { "muted".to_string() }
            else { format!("{}%", (widget.player.volume() * 100.0).round()) };
        let pos = widget.player.pos();
        let dur = widget.player.duration();

        // Draw play info
        let play_info_rect = Text::new(format!("{} / {}  {}", pos.to_readable(), dur.to_readable(), volume), style)
            .align(Align::End)
            .draw(buf, rect);

        let track_info_rect = rect.margin_right(play_info_rect.width + 2);

        // Draw track info
        if let Some(artist) = track.try_artist() {
            Text::new(format!("{}  {} - {}", playstate, title, artist), style)
                .clip(Clip::Ellipsis)
                .draw(buf, track_info_rect)
        } else {
            Text::new(format!("{}  {}", playstate, title), style)
                .clip(Clip::Ellipsis)
                .draw(buf, track_info_rect)
        };
    } else {
        // Draw something else...
        Text::new("There should be some smart quote... - Unknown man", widget.config.theme.player.stopped)
            .italic()
            .clip(Clip::Ellipsis)
            .draw(buf, rect);
    }

    rect
}
fn draw_progress(widget: &PlayerWidget, style: Style, buf: &mut Buffer, rect: Rect) -> Rect {
    let pos = widget.player.pos();
    let dur = widget.player.duration();
    let progress = pos.as_secs() as f32 / dur.as_secs() as f32;

    // Draw audio progress
    Progress::new(progress)
        .with_style(style)
        .with_char(widget.config.format.progress.to_string())
        .with_track_char(widget.config.format.progress_track.to_string())
        .with_thumb(&widget.config.format.progress_thumb)
        .draw(buf, rect);

    rect
}
fn draw_classic(widget: &PlayerWidget, style: Style, buf: &mut Buffer, rect: Rect) -> Rect {
    let is_reversed = widget.style == PlayerStyle::ClassicReverse;

    let text_rect =
        if is_reversed { rect }
        else { rect.margin_top(1) };
    let progress_rect =
        if is_reversed { rect.margin_top(1) }
        else { rect };

    draw_info(widget, style, buf, text_rect);
    draw_progress(widget, style, buf, progress_rect);

    rect
}
