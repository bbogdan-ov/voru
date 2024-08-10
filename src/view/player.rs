use tuich::{buffer::Buffer, layout::Rect, widget::RefDraw};

use crate::{config::Config, player::Player, widget::PlayerWidget};

#[derive(Debug)]
pub struct PlayerView {}
impl PlayerView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, config: &Config, player: &Player, buf: &mut Buffer, rect: Rect) -> Rect {
        let max_width =
            if config.layout.player_max_width == 0 { rect.width }
            else { config.layout.player_max_width + 2 };
        let player_rect = PlayerWidget::style_rect(rect, config.style.player)
            .min_size((max_width, rect.height))
            .align_center(rect);

        PlayerWidget {
            config,
            player,
            style: config.style.player
        }.draw(buf, player_rect);

        player_rect
    }
}
