use std::borrow::Cow;

use tuich::{buffer::Buffer, layout::{Point, Rect}, style::{Color, Style}, unicode_width::UnicodeWidthStr, widget::Draw};

#[derive(Debug, Clone)]
pub struct Progress<'a> {
    value: f32,
    style: Style,
    track_style: Style,
    char: Cow<'a, str>,
    track_char: Cow<'a, str>,
    thumb: Option<Cow<'a, str>>
}
impl<'a> Progress<'a> {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            style: Color::Green.into(),
            track_style: Color::LightBlack.into(),
            char: "─".into(),
            track_char: "─".into(),
            thumb: None
        }
    }

    pub fn with_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
    pub fn with_track_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.track_style = style.into();
        self
    }
    pub fn with_char<C: Into<Cow<'a, str>>>(mut self, char: C) -> Self {
        self.char = char.into();
        self
    }
    pub fn with_track_char<C: Into<Cow<'a, str>>>(mut self, char: C) -> Self {
        self.track_char = char.into();
        self
    }
    pub fn with_thumb<T: Into<Cow<'a, str>>>(mut self, thumb: T) -> Self {
        self.thumb = Some(thumb.into());
        self
    }
}
impl<'a> Draw for Progress<'a> {
    fn draw(self, buf: &mut Buffer, rect: Rect) -> Rect {
        let rect = rect.with_height(1);

        let width = rect.width as f32;
        for i in 0..rect.width {
            let is_filled = i as f32 / width < self.value;
            let color =
                if is_filled { self.style }
                else { self.track_style };
            let char =
                if is_filled { &self.char }
                else { &self.track_char };

            buf.set(
                rect.pos().add((i, 0)),
                Some(char.as_ref()),
                color
            );
        }

        // Draw thumb
        if let Some(thumb) = self.thumb.as_ref().map(|t| t.as_ref()) {
            let thumb_width = thumb.width() as u16;
            let thumb_max_x = rect.width.saturating_sub(thumb_width);
            let thumb_x = ((thumb_max_x + 1) as f32 * self.value).round() as u16;
            let thumb_x = thumb_x.min(thumb_max_x);

            buf.set_string(
                rect.pos().add((thumb_x, 0)),
                0,
                thumb,
                self.style
            );
        }

        rect
    }
}
