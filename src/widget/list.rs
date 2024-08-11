use std::borrow::Cow;

use tuich::{
    buffer::Buffer,
    event::Key,
    layout::{Clip, Rect},
    style::Style,
    text::Text,
    widget::{Clear, Draw, RefDraw},
};

use crate::{app::AppContext, match_keys};

/// List event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListEvent {
    Nope,
    Selected(usize),
    Chosen(usize),
    Scrolled(u16)
}

/// List state
#[derive(Debug, Clone)]
pub struct ListState {
    current: usize,
    scroll: u16,
    scrolloff: u16,
    pub active: bool,

    /// Visible height of the list
    height: u16,
    /// Visible height of a list item
    item_height: u16,
    /// Items count
    count: usize,
}
impl ListState {
    pub fn new() -> Self {
        Self {
            current: 0,
            scroll: 0,
            scrolloff: 2,
            active: true,

            height: 0,
            item_height: 1,
            count: 0,
        }
    }

    pub fn with_scrolloff(mut self, value: u16) -> Self {
        self.scrolloff = value;
        self
    }

    pub fn handle_key(&mut self, ctx: &AppContext, key: Key) -> ListEvent {
        match_keys! {
            ctx.config, key,

            choose_item => ListEvent::Chosen(self.current()),
            select_next_item => self.select_next(1),
            select_prev_item => self.select_prev(1),
            select_next_item_fast => self.select_next(ctx.config.fast_jump),
            select_prev_item_fast => self.select_prev(ctx.config.fast_jump),
            select_next_item_super_fast => self.select_next(ctx.config.super_fast_jump),
            select_prev_item_super_fast => self.select_prev(ctx.config.super_fast_jump),
            select_first_item => self.select_first(),
            select_last_item => self.select_last();

            else { ListEvent::Nope }
        }
    }
    
    pub fn select(&mut self, index: usize) -> ListEvent {
        let index = index.clamp(0, self.count.saturating_sub(1));
        self.current = index;

        // Scrolling
        let cur = self.current as u16;
        let scroll_top = self.scroll_top();
        let scroll_bottom = self.scroll_bottom();

        // Scrolling
        if cur >= scroll_bottom {
            self.scroll_down(cur.saturating_sub(scroll_bottom));
        } else if cur <= scroll_top {
            self.scroll_up(scroll_top.saturating_sub(cur));
        }

        ListEvent::Selected(index)
    }
    pub fn select_next(&mut self, jump: usize) -> ListEvent {
        self.select(self.current + jump)
    }
    pub fn select_prev(&mut self, jump: usize) -> ListEvent {
        self.select(self.current.saturating_sub(jump))
    }
    pub fn select_first(&mut self) -> ListEvent {
        self.select(0)
    }
    pub fn select_last(&mut self) -> ListEvent {
        self.select(self.count)
    }

    pub fn set_scroll(&mut self, scroll: u16) -> ListEvent {
        let scroll = scroll.clamp(0, self.scroll_height());
        self.scroll = scroll;
        ListEvent::Scrolled(scroll)
    }
    pub fn scroll_up(&mut self, speed: u16) -> ListEvent {
        self.set_scroll(self.scroll.saturating_sub(speed))
    }
    pub fn scroll_down(&mut self, speed: u16) -> ListEvent {
        self.set_scroll(self.scroll + speed)
    }

    pub fn current(&self) -> usize { self.current }
    pub fn count(&self) -> usize { self.count }
    pub fn height(&self) -> u16 { self.height }
    pub fn scroll(&self) -> u16 { self.scroll }
    pub fn scroll_top(&self) -> u16 {
        self.scroll + self.scrolloff
    }
    pub fn scroll_bottom(&self) -> u16 {
        let height =
            if self.item_height == 1 { self.height }
            else { (self.height as f32 / self.item_height as f32).round() as u16 };
        self.scroll + height.saturating_sub(self.scrolloff + 1)
    }
    pub fn scroll_height(&self) -> u16 {
        (self.count as u16).saturating_sub(self.height / self.item_height)
    }
}

/// List widget
#[derive(Debug)]
pub struct List<'a, T> {
    state: &'a mut ListState,
    items: &'a Vec<T>,
    item_height: u16,
}
impl<'a, T> List<'a, T> {
    pub fn new(state: &'a mut ListState, items: &'a Vec<T>) -> Self {
        Self {
            state,
            items,
            item_height: 1
        }
    }

    pub fn with_item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    pub fn draw<F: Fn(usize, &T, &mut ListState, &mut Buffer, Rect) -> Rect>(&mut self, buf: &mut Buffer, rect: Rect, draw_item: F) -> Rect {
        let mut height = 0u16;

        self.state.count = self.items.len();
        self.state.height = rect.height;
        self.state.item_height = self.item_height;

        if self.state.current >= self.state.count {
            self.state.select(self.state.current);
        }

        for (index, item) in self.items.iter().enumerate() {
            // Skip item drawing if its index is less than scroll offset
            if (index as u16) < self.state.scroll() { continue; }
            // Break if items are overflow
            if height > rect.height.saturating_sub(1) { break; }

            let item_rect = rect
                .add_y(height)
                .with_height(self.item_height);

            draw_item(index, item, self.state, buf, item_rect);

            height += item_rect.height;
        }

        rect.with_height(height)
    }
}

/// List item widget
#[derive(Debug)]
pub struct ListItem<'a> {
    index: usize,
    state: &'a ListState,
    content: Cow<'a, str>,
    style: Style,
    cur_style: Style,
}
impl<'a> ListItem<'a> {
    pub fn new<C: Into<Cow<'a, str>>>(index: usize, state: &'a ListState, content: C) -> Self {
        Self {
            index,
            state,
            content: content.into(),
            style: Style::default(),
            cur_style: Style::default().reverse(true)
        }
    }

    pub fn with_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
    pub fn with_cur_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.cur_style = style.into();
        self
    }
}
impl<'a> RefDraw for ListItem<'a> {
    fn draw(&self, buf: &mut Buffer, rect: Rect) -> Rect {
        let rect = rect.with_height(1);
        let is_cur = self.state.active && self.index == self.state.current();
        let style =
            if is_cur { self.cur_style }
            else { self.style };

        Clear::new(style)
            .draw(buf, rect);

        Text::new(self.content.as_ref(), ())
            .clip(Clip::Ellipsis)
            .draw(buf, rect.margin((1, 0)));

        rect
    }
}
