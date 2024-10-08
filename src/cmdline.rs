use tuich::{
    buffer::Buffer,
    event::Key,
    layout::{Clip, Rect},
    text::Text,
    widget::{prompt::PromptState, Clear, Draw, Prompt},
};

use crate::{
    app::{AppContext, UpdateError},
    commands::{exec_command, Cmd},
    match_keys,
    Action,
};

/// Command line
#[derive(Debug)]
pub struct CmdLine {
    pub state: PromptState,
    history: Vec<String>,
    cur_history_item: Option<usize>,
    is_completing: bool
}
impl CmdLine {
    pub fn new() -> Self {
        Self {
            state: PromptState::default(),
            history: vec![],
            cur_history_item: None,
            is_completing: false
        }
    }

    pub fn handle_key(
        &mut self,
        ctx: &mut AppContext,
        key: Key,
    ) -> Result<Action, UpdateError> {
        match_keys! {
            ctx.config, key,

            enter => return self.execute(ctx),
            escape => {
                if self.is_completing {
                    // Don't exit if completion was enabled, just turn it off
                    self.is_completing = false;
                } else {
                    self.exit(ctx)
                }
            },
            complete => self.is_completing = true,
            next_history => self.next_history(),
            prev_history => self.prev_history();

            else { self.state.handle_keys(key); }
        }

        Ok(Action::Draw)
    }

    fn execute(&mut self, ctx: &mut AppContext) -> Result<Action, UpdateError> {
        let value = self.value().trim().to_string();

        // Just exit if the value is empty
        if value.is_empty() {
            self.exit(ctx);
            return Ok(Action::Draw);
        }

        // Execute the command
        let result = exec_command(ctx, &value);

        // Save the command to the history and remove old duplicate
        if let Some(dup_index) = self.history.iter().position(|i| i.eq(&value)) {
            self.history.remove(dup_index);
        }
        self.history.push(value);

        self.exit(ctx);
        result
    }
    fn exit(&mut self, ctx: &mut AppContext) {
        self.state.clear();
        self.is_completing = false;
        self.cur_history_item = None;
        ctx.state.enter_mode(crate::app::Mode::Normal);
    }

    fn next_history(&mut self) {
        let history_len = self.history.len();
        if history_len == 0 { return; }
        let Some(cur_item) = self.cur_history_item else {
            return;
        };
        let cur_item = cur_item + 1;

        if cur_item >= history_len {
            self.state.clear();
            self.cur_history_item = None;
        } else {
            self.state.set_value(self.history[cur_item].clone());
            self.cur_history_item = Some(cur_item);
        }

        self.state.move_end();
    }
    fn prev_history(&mut self) {
        let history_len = self.history.len();
        if history_len == 0 { return; }
        let cur_item = match self.cur_history_item {
            Some(cur_item) => cur_item.saturating_sub(1),
            None => history_len.saturating_sub(1)
        };

        self.state.set_value(self.history[cur_item].clone());
        self.state.move_end();
        self.cur_history_item = Some(cur_item);
    }

    pub fn draw(&self, ctx: &AppContext, buf: &mut Buffer, rect: Rect) -> Rect {
        let prompt_rect = rect.with_height(1);

        Clear::new(ctx.config.theme.cmdline)
            .draw(buf, prompt_rect);

        // Draw colon (:)
        buf.set(prompt_rect.pos(), Some(":"), ());

        // Draw prompt
        Prompt::new(&self.state)
            .style(ctx.config.theme.cmdline)
            .draw(buf, prompt_rect.margin_left(1));

        // Draw completion
        if self.is_completing {
            self.draw_completion(ctx, buf, prompt_rect);
        }

        prompt_rect
    }
    fn draw_completion(&self, ctx: &AppContext, buf: &mut Buffer, prompt_rect: Rect) {
        let value = self.value().trim_start();
        if value.contains(' ') || value.is_empty() {
            return;
        }

        let mut compl_height = 0_u16;
        for cmd in &ctx.commands.list {
            let alias = match cmd {
                Cmd::Normal(_, _) => None,
                Cmd::Alias(_, _, to) => Some(to)
            };

            let name = cmd.name();

            if !name.contains(value) && !alias.is_some_and(|a| a.contains(value)) {
                continue;
            }

            let kind = cmd.kind();
            let args = kind.args();

            let name = match args {
                Some(args) => format!("{} {}", name, args),
                None => name.to_string()
            };
            let desc = kind.description();
            let desc = match alias {
                Some(alias) => format!("(alias to :{alias}) {desc}"),
                None => desc.to_string()
            };

            let item_rect = prompt_rect.add_y(compl_height + 1);
            let text_rect = item_rect.margin((1, 0));

            let style =
                if alias.is_some() { ctx.config.theme.completion_alias }
                else { ctx.config.theme.completion };

            Clear::new(style)
                .draw(buf, item_rect);

            let name_rect = Text::from(name)
                .draw(buf, text_rect);
            Text::from(desc)
                .clip(Clip::Ellipsis)
                .draw(buf, text_rect.margin_left(name_rect.width.max(35)));

            compl_height += 1;
        }
    }

    // Get

    pub fn value(&self) -> &String {
        self.state.value()
    }
}
