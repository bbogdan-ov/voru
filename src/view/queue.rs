use tuich::{
    buffer::Buffer,
    event::Key,
    layout::Rect,
    widget::Draw,
};

use crate::{
    app::AppContext,
    match_keys,
    player::{PlaybackError, PlaybackResult},
    traits::ToReadable,
    widget::{List, ListState, TrackTable, TrackWidget, ViewWidget},
    Action,
};

/// Queue view
#[derive(Debug)]
pub struct QueueView {
    list_state: ListState
}
impl QueueView {
    pub fn new() -> Self {
        Self {
            list_state: ListState::new()
        }
    }

    pub fn handle_key(&mut self, ctx: &mut AppContext, key: Key) -> Result<Action, PlaybackError> {
        match_keys! {
            ctx.config, key,

            play => ctx.player.play(self.cur_track())?,
            queue_focus => self.focus(ctx),
            queue_move_up => self.move_up(ctx, 1)?,
            queue_move_down => self.move_down(ctx, 1)?,
            queue_remove => ctx.player.queue_remove(self.cur_track())?;

            else {
                return Ok(self.list_state.handle_key(ctx, key).into())
            }
        }

        Ok(Action::Draw)
    }

    fn focus(&mut self, ctx: &AppContext) {
        if let Some(index) = ctx.player.cur_track_index {
            self.list_state.select(index);
        }
    }
    fn move_up(&mut self, ctx: &mut AppContext, jump: usize) -> PlaybackResult {
        let cur = self.list_state.current();
        let new_index = cur.saturating_sub(jump);
        ctx.player.queue_move_to(cur, new_index)?;
        self.list_state.select(new_index);
        Ok(())
    }
    fn move_down(&mut self, ctx: &mut AppContext, jump: usize) -> PlaybackResult {
        let cur = self.list_state.current();
        let new_index = cur + jump;
        ctx.player.queue_move_to(cur, new_index)?;
        self.list_state.select(new_index);
        Ok(())
    }

    pub fn draw(&mut self, ctx: &AppContext, buf: &mut Buffer, rect: Rect) -> Rect {
        let playstate = ctx.player.playstate();
        let tracks_count = ctx.player.queue.len();
        let queue_dur = ctx.player.queue_dur.to_readable();

        let desc = if let Some(cur_index) = ctx.player.cur_track_index {
            let elapsed = (ctx.player.elapsed + ctx.player.pos()).to_readable();
            format!("{} / {} tracks  {} / {}", cur_index + 1, tracks_count, elapsed, queue_dur)
        } else {
            format!("{} tracks  {}", tracks_count, queue_dur)
        };

        let content_rect = ViewWidget::new(&ctx.config, playstate, "Queue")
            .with_desc(desc)
            .draw(buf, rect);

        let table = TrackTable::new(tracks_count, content_rect);

        List::new(&mut self.list_state, &ctx.player.queue)
            .draw(buf, content_rect, |index, track, list_state, buf, rect| {
                TrackWidget {
                    index,
                    state: list_state,
                    ctx,
                    track,
                    playing: ctx.player.is_track_index_current(&index)
                }.draw(&table, buf, rect)
            });

        rect
    }

    // Get

    fn cur_track(&self) -> usize {
        self.list_state.current()
    }
}
