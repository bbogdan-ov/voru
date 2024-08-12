use std::borrow::Cow;

use tuich::{buffer::Buffer, layout::{Align, Clip, Rect}, style::Color, text::Text, widget::Draw};

use crate::{config::Config, player::PlayState};

/// View widget
#[derive(Debug)]
pub struct ViewWidget<'a> {
    config: &'a Config,
    playstate: PlayState,
    title: Cow<'a, str>,
    desc: Option<Cow<'a, str>>,
    active: bool
}
impl<'a> ViewWidget<'a> {
    pub fn new<T: Into<Cow<'a, str>>>(config: &'a Config, playstate: PlayState, title: T) -> Self {
        Self {
            config,
            playstate,
            title: title.into(),
            desc: None,
            active: true
        }
    }

    pub fn with_desc<D: Into<Cow<'a, str>>>(mut self, desc: D) -> Self {
        self.desc = Some(desc.into());
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}
impl<'a> Draw for ViewWidget<'a> {
    fn draw(self, buf: &mut Buffer, rect: Rect) -> Rect {
        let is_playing = self.playstate == PlayState::Playing;
        let title_style =
            if self.active && is_playing { self.config.theme.title_active_playing }
            else if self.active && !is_playing { self.config.theme.title_active_paused }
            else { self.config.theme.title_inactive };

        let header_rect = rect.margin((1, 0)).with_height(1);

        // Draw description
        let desc_width = if let Some(desc) = self.desc {
            Text::new(desc, Color::Gray)
                .align(Align::End)
                .clip(Clip::Ellipsis)
                .draw(buf, header_rect)
                .width
        } else { 0 };

        // Draw title
        Text::new(self.title, title_style)
            .clip(Clip::Ellipsis)
            .draw(buf, header_rect.margin_right(desc_width + 4));

        rect.margin_top(header_rect.height + 1)
    }
}
