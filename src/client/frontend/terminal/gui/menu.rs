use super::super::gui::util::{self, Context};

use crate::client::state::{State};
use crate::client::util::store::{StateManager};

use tui::{Frame};
use tui::backend::{CrosstermBackend};
use tui::widgets::{Block, Borders, BorderType, Paragraph, Wrap};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use std::io::{self, Stdout};

const MAIN_TITLE: &'static str = concat!(
r"   _____                .__.__   _____                                ", "\n",
r"  /  _  \   ______ ____ |__|__| /  _  \_______   ____   ____ _____    ", "\n",
r" /  /_\  \ /  ___// ___\|  |  |/  /_\  \_  __ \_/ __ \ /    \\__  \   ", "\n",
r"/    |    \\___ \\  \___|  |  /    |    \  | \/\  ___/|   |  \/ __ \_ ", "\n",
r"\____|__  /______>\_____>__|__\____|__  /__|    \_____>___|__(______/ ", "\n",
r"        \/                            \/                              ", "\n",
);
pub const DIMENSION: (u16, u16) = (70, 20);

/*
    Server address:   127.0.0.1:3000                   (Connected!)
    Player name:      L                              Capital letter

    --Server Info----------    -- Waiting room ---------------------
    | Players: 4          |    | Player 1: L                       |
    | Map size: 30x30     |    | Player 2: T                       |
    | Winner points: 15   |    | Player 3: E                       |
    | UDP port: 3456      |    | Player 4: (waiting...)            |
    -----------------------    -------------------------------------

                         Initializing Arena...
*/


pub struct Menu {
}

impl Menu {
    pub fn new() -> Menu {
        Menu {}
    }

    pub fn draw(&mut self, ctx: &mut Context, space: Rect) {
        let gui_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(MAIN_TITLE.chars().filter(|&c| c == '\n').count() as u16 + 2),
                Constraint::Length(4),
                Constraint::Percentage(100),
            ].as_ref())
            .split(space);

        self.draw_menu_panel(ctx, gui_layout[0]);
        self.draw_input_panel(ctx, gui_layout[1]);

        let panel_layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Min(2),
                Constraint::Percentage(40),
            ].as_ref())
            .split(gui_layout[2]);

        self.draw_server_info_panel(ctx, panel_layout[0]);
        self.draw_waiting_room_panel(ctx, panel_layout[2]);
    }

    fn draw_menu_panel(&self, ctx: &mut Context, space: Rect) {
        let main_title = Paragraph::new(MAIN_TITLE)
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        ctx.frame.render_widget(main_title, space);
    }

    fn draw_input_panel(&self, ctx: &mut Context, space: Rect) {
        let input_value = Paragraph::new("Server address\n Player name\n")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        ctx.frame.render_widget(input_value, space);
    }

    fn draw_server_info_panel(&self, ctx: &mut Context, space: Rect) {
        let server_info_panel = Paragraph::new("Players:")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    "Server info",
                    Style::default().add_modifier(Modifier::BOLD)
                )))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        ctx.frame.render_widget(server_info_panel, space);
    }

    fn draw_waiting_room_panel(&self, ctx: &mut Context, space: Rect) {
        let waiting_room_panel = Paragraph::new("Player 1:")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Waiting room",
                    Style::default().add_modifier(Modifier::BOLD)
                )))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        ctx.frame.render_widget(waiting_room_panel, space);
    }
}
