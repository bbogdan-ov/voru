use tuich::{
    buffer::Buffer,
    event::Key,
    layout::Rect,
    widget::Draw,
};

use crate::{
    config::Config,
    match_keys,
    player::{PlaybackError, PlaybackResult, Player},
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

    pub fn handle_key(&mut self, config: &Config, player: &mut Player, key: Key) -> Result<Action, PlaybackError> {
        match_keys! {
            config, key,

            play => player.play(self.cur_track())?,
            queue_focus => self.focus(player),
            queue_move_up => self.move_up(player, 1)?,
            queue_move_down => self.move_down(player, 1)?,
            queue_remove => player.queue_remove(self.cur_track())?;

            else {
                return Ok(self.list_state.handle_key(config, key).into())
            }
        }

        Ok(Action::Draw)
    }

    fn focus(&mut self, player: &Player) {
        if let Some(index) = player.cur_track_index {
            self.list_state.select(index);
        }
    }
    fn move_up(&mut self, player: &mut Player, jump: usize) -> PlaybackResult {
        let cur = self.list_state.current();
        let new_index = cur.saturating_sub(jump);
        player.queue_move_to(cur, new_index)?;
        self.list_state.select(new_index);
        Ok(())
    }
    fn move_down(&mut self, player: &mut Player, jump: usize) -> PlaybackResult {
        let cur = self.list_state.current();
        let new_index = cur + jump;
        player.queue_move_to(cur, new_index)?;
        self.list_state.select(new_index);
        Ok(())
    }

    pub fn draw(&mut self, config: &Config, player: &Player, buf: &mut Buffer, rect: Rect) -> Rect {
        let playstate = player.playstate();
        let tracks_count = player.queue.len();
        let queue_dur = player.queue_dur.to_readable();

        let desc = if let Some(cur_index) = player.cur_track_index {
            let elapsed = (player.elapsed + player.pos()).to_readable();
            format!("{} / {} tracks  {} / {}", cur_index + 1, tracks_count, elapsed, queue_dur)
        } else {
            format!("{} tracks  {}", tracks_count, queue_dur)
        };

        let content_rect = ViewWidget::new(config, playstate, "Queue")
            .with_desc(desc)
            .draw(buf, rect);

        let table = TrackTable::new(tracks_count, content_rect);

        List::new(&mut self.list_state, &player.queue)
            .draw(buf, content_rect, |index, track, list_state, buf, rect| {
                TrackWidget {
                    index,
                    config,
                    state: list_state,
                    playstate,
                    track,
                    playing: player.is_track_index_current(&index)
                }.draw(&table, buf, rect)
            });

        rect
    }

    // Get

    fn cur_track(&self) -> usize {
        self.list_state.current()
    }
}
