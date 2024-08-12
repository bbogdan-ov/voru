use serde::{Deserialize, Serialize};
use tuich::{buffer::Buffer, layout::{Align, Clip, Rect}, style::{Style, Stylized}, text::Text, widget::{Draw, RefDraw}};

use crate::{app::AppContext, player::PlayState, traits::ToReadable};

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
    pub ctx: &'a AppContext,
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
        let theme = &self.ctx.config.theme;

        let rect = Self::style_rect(rect, self.style);

        let playstate = self.ctx.player.playstate();
        let style = match playstate {
            PlayState::Stopped => theme.player_stopped,
            PlayState::Playing => theme.player_playing,
            _ => theme.player_paused
        };

        match self.style {
            PlayerStyle::Classic |
            PlayerStyle::ClassicReverse => draw_classic(self, self.ctx, style, buf, rect),

            PlayerStyle::Progress => draw_progress(self.ctx, style, buf, rect),

            PlayerStyle::Text => draw_info(self.ctx, style, buf, rect),
        }
    }
}

// Draw styles
fn draw_info(ctx: &AppContext, style: Style, buf: &mut Buffer, rect: Rect) -> Rect {
    let rect = rect.with_height(1);

    if let Some(track) = &ctx.player.cur_track {
        let playstate = ctx.player.playstate();
        let title = track.title();
        let volume =
            if ctx.player.muted() { "muted".to_string() }
            else { format!("{}%", (ctx.player.volume() * 100.0).round()) };
        let pos = ctx.player.pos();
        let dur = ctx.player.duration();

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
        Text::new("There should be some smart quote... - Unknown man", ctx.config.theme.player_stopped)
            .italic()
            .clip(Clip::Ellipsis)
            .draw(buf, rect);
    }

    rect
}
fn draw_progress(ctx: &AppContext, style: Style, buf: &mut Buffer, rect: Rect) -> Rect {
    let pos = ctx.player.pos();
    let dur = ctx.player.duration();
    let progress = pos.as_secs() as f32 / dur.as_secs() as f32;

    // Draw audio progress
    Progress::new(progress)
        .with_style(style)
        .with_char(ctx.config.format.progress.to_string())
        .with_track_char(ctx.config.format.progress_track.to_string())
        .with_thumb(&ctx.config.format.progress_thumb)
        .draw(buf, rect);

    rect
}
fn draw_classic(widget: &PlayerWidget, ctx: &AppContext, style: Style, buf: &mut Buffer, rect: Rect) -> Rect {
    let is_reversed = widget.style == PlayerStyle::ClassicReverse;

    let text_rect =
        if is_reversed { rect }
        else { rect.margin_top(1) };
    let progress_rect =
        if is_reversed { rect.margin_top(1) }
        else { rect };

    draw_info(ctx, style, buf, text_rect);
    draw_progress(ctx, style, buf, progress_rect);

    rect
}
