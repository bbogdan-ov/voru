use tuich::{buffer::Buffer, layout::Rect, widget::RefDraw};

use crate::{app::AppContext, widget::PlayerWidget};

#[derive(Debug)]
pub struct PlayerView {}
impl PlayerView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, ctx: &AppContext, buf: &mut Buffer, rect: Rect) -> Rect {
        let max_width =
            if ctx.config.layout.player_max_width == 0 { rect.width }
            else { ctx.config.layout.player_max_width + 2 };
        let player_rect = PlayerWidget::style_rect(rect, ctx.config.style.player)
            .min_size((max_width, rect.height))
            .align_center(rect);

        PlayerWidget {
            ctx,
            style: ctx.config.style.player
        }.draw(buf, player_rect);

        player_rect
    }
}
