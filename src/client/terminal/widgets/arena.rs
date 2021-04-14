use super::util::{self};

use crate::client::state::{State, GameStatus, Player};
use crate::client::store::{Store, Action};
use crate::client::terminal::input::{InputEvent};
use crate::client::configuration::{Config};

use crate::direction::{Direction};
use crate::character::{CharacterId, Character};
use crate::message::{EntityData};
use crate::ids::{SkillId, EntityId};

use tui::buffer::{Buffer};
use tui::widgets::{Paragraph, Block, Borders, BorderType, Widget};
use tui::layout::{Layout, Constraint, Direction as Dir, Rect, Alignment, Margin};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use crossterm::event::{KeyCode};

use std::time::{Instant, Duration};
use std::collections::{HashMap};

pub struct Arena {
    previous_entities: HashMap<EntityId, EntityData>,
    damaged_entities: HashMap<EntityId, Instant>,
}

impl Arena {
    pub fn new(_config: &Config) -> Arena {
        Arena { previous_entities: HashMap::new(), damaged_entities: HashMap::new() }
    }

    pub fn process_event(&mut self, store: &mut Store, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(key_event) => match key_event.code {
                KeyCode::Enter => {
                    if let GameStatus::Finished = store.state().server.game.status {
                        store.dispatch(Action::CloseGame);
                    }
                }
                KeyCode::Char(c) => {
                    if let GameStatus::Started = store.state().server.game.status {
                        match c {
                            'w' => store.dispatch(Action::MovePlayer(Direction::Up)),
                            'a' => store.dispatch(Action::MovePlayer(Direction::Left)),
                            's' => store.dispatch(Action::MovePlayer(Direction::Down)),
                            'd' => store.dispatch(Action::MovePlayer(Direction::Right)),
                            ' ' => store.dispatch(Action::CastSkill(SkillId(1))),
                            _ => (),
                        }
                    }
                }
                _ => (),
            },
            InputEvent::ResizeDisplay(..) => {}
        }
    }

    pub fn update(&mut self, state: &State) {
        let arena = state.server.game.arena();

        const ENTITY_DAMAGE_ANIMATION_TIME: Duration = Duration::from_millis(66);
        let now = Instant::now();
        self.damaged_entities.retain(|_, from| now - *from < ENTITY_DAMAGE_ANIMATION_TIME);

        for (id, entity) in &self.previous_entities {
            let damaged = entity.health > arena.entities.get(id).map(|e| e.health).unwrap_or(0);

            if damaged {
                self.damaged_entities.insert(*id, now);
            }
        }
        self.previous_entities = arena.entities.clone();
    }
}

#[derive(derive_new::new)]
pub struct ArenaWidget<'a> {
    state: &'a State,
    arena: &'a Arena,
}

impl<'a> ArenaWidget<'a> {
    pub fn dimension(state: &State) -> (u16, u16) {
        let map_size = state.server.game_info.as_ref().unwrap().map_size as u16;
        let map_dim = MapWidget::dimension(map_size);

        (
            PlayerPanelListWidget::WIDTH + 1 + map_dim.0,
            1 + ArenaInfoLabelWidget::HEIGHT + map_dim.1 + NotificationLabelWidget::HEIGHT,
        )
    }
}

impl Widget for ArenaWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let map_size = self.state.server.game_info.as_ref().unwrap().map_size as u16;
        let map_dim = MapWidget::dimension(map_size);

        let column = Layout::default()
            .direction(Dir::Vertical)
            .constraints(
                [
                    Constraint::Length(1), //Margin
                    Constraint::Length(ArenaInfoLabelWidget::HEIGHT),
                    Constraint::Length(map_dim.1),
                    Constraint::Length(NotificationLabelWidget::HEIGHT),
                ]
                .as_ref(),
            )
            .split(area);

        ArenaInfoLabelWidget::new(self.state).render(column[1], buffer);

        let row = Layout::default()
            .direction(Dir::Horizontal)
            .constraints(
                [
                    Constraint::Length(PlayerPanelListWidget::WIDTH),
                    Constraint::Length(1), //Margin
                    Constraint::Length(map_dim.0),
                ]
                .as_ref(),
            )
            .split(column[2]);

        PlayerPanelListWidget::new(self.state).render(row[0], buffer);

        MapWidget::new(self.state, self.arena).render(row[2], buffer);

        NotificationLabelWidget::new(self.state).render(column[3], buffer);
    }
}

#[derive(derive_new::new)]
struct ArenaInfoLabelWidget<'a> {
    state: &'a State,
}

impl ArenaInfoLabelWidget<'_> {
    pub const HEIGHT: u16 = 1;
}

impl Widget for ArenaInfoLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let points = self.state.server.game_info.as_ref().unwrap().winner_points;
        let number = self.state.server.game.arena_number;

        let title = Spans::from(vec![
            Span::raw("Arena "),
            Span::styled(number.to_string(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" · Points to win: "),
            Span::styled(points.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(title).alignment(Alignment::Center).render(area, buffer);
    }
}

#[derive(derive_new::new)]
struct PlayerPanelListWidget<'a> {
    state: &'a State,
}

impl PlayerPanelListWidget<'_> {
    pub const WIDTH: u16 = PlayerPanelWidget::DIMENSION.0;
}

impl Widget for PlayerPanelListWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let players = &self.state.server.game.players;

        let mut constraints = vec![Constraint::Length(1)]; //Top margin
        constraints
            .extend(players.iter().map(|_| Constraint::Length(PlayerPanelWidget::DIMENSION.1)));
        constraints.push(Constraint::Min(0)); // Bottom margin

        let row = Layout::default().direction(Dir::Vertical).constraints(constraints).split(area);

        for (index, player) in self.state.server.game.players.iter().enumerate() {
            let character = &self.state.server.game.characters[&player.character_id];
            let entity = self.state.server.game.arena().entities.get(&player.entity_id);

            PlayerPanelWidget::new(self.state, player, character, entity)
                .render(row[index + 1], buffer)
        }
    }
}

#[derive(derive_new::new)]
struct PlayerPanelWidget<'a> {
    state: &'a State,
    player: &'a Player,
    character: &'a Character,
    entity: Option<&'a EntityData>,
}

impl<'a> PlayerPanelWidget<'a> {
    pub const DIMENSION: (u16, u16) = (27, 5);
}

impl Widget for PlayerPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let is_user = self.player.id == self.state.server.game.arena().user_player.player_id;
        let box_border_style = match is_user {
            true => Style::default()
                .fg(if self.entity.is_some() { Color::White } else { Color::DarkGray })
                .add_modifier(Modifier::BOLD),
            false => Style::default().fg(if self.entity.is_some() {
                Color::Gray
            }
            else {
                Color::DarkGray
            }),
        };

        // Symbol panel
        let symbol_area = Rect::new(area.x, area.y, 5, 3).intersection(area);
        Block::default()
            .borders(Borders::ALL)
            .border_style(box_border_style)
            .border_type(BorderType::Rounded)
            .render(symbol_area, buffer);

        let player_color = Color::White;
        let player_style = match is_user {
            true => Style::default().fg(player_color).add_modifier(Modifier::BOLD),
            false => Style::default().fg(player_color),
        };

        let symbol = self.character.symbol().to_string();
        buffer.set_string(area.x + 2, area.y + 1, symbol, player_style);

        // Main panel
        let panel_area = Rect::new(symbol_area.right(), area.y, 22, 4).intersection(area);
        let points = self.player.points;
        let points_style = Style::default().fg(Color::White);
        Block::default()
            .title(Spans::from(vec![
                Span::raw("──"),
                Span::styled(" Pts: ", points_style),
                Span::styled(points.to_string(), points_style),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_style(box_border_style)
            .border_type(BorderType::Rounded)
            .render(panel_area, buffer);

        // Bars
        let content = panel_area.inner(&Margin { vertical: 1, horizontal: 1 });

        let bar_area = Rect::new(content.x, content.y, content.width, 1).intersection(area);
        let health = self.entity.map(|e| e.health).unwrap_or(0);
        BarWidget::new(health, self.character.max_health(), Color::Green).render(bar_area, buffer);

        let bar_area = Rect::new(content.x, content.y + 1, content.width, 1).intersection(area);
        let energy = self.entity.map(|e| e.energy).unwrap_or(0);
        BarWidget::new(energy, self.character.max_energy(), Color::Cyan).render(bar_area, buffer);

        // Bottom
        let arrow = box_border_style;
        let bottom = Rect::new(panel_area.x, panel_area.bottom() - 1, panel_area.width, 1)
            .intersection(area);
        let bottom = bottom.inner(&Margin { vertical: 0, horizontal: 2 });

        let clean_row = (0..bottom.width).map(|_| " ").collect::<String>();
        buffer.set_string(bottom.x, bottom.y, clean_row, arrow);

        buffer.set_string(bottom.x, bottom.y, ">", arrow);
        buffer.set_string(bottom.right() - 1, bottom.y, "<", arrow);
    }
}

#[derive(derive_new::new)]
struct BarWidget {
    current: usize,
    max: usize,
    color: Color,
}

impl Widget for BarWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let bar_style = Style::default().fg(self.color);
        let bar_off_style = Style::default().fg(self.color);

        let bar_len = 10 as u16;
        let current_len = ((self.current + 9) as f32 / bar_len as f32) as usize;
        let bar = (0..bar_len as usize)
            .map(|index| {
                if index + 1 == current_len {
                    let partial = self.current as u16 % bar_len;
                    if partial > bar_len / 2 || partial == 0 {
                        Span::styled("=", bar_style.add_modifier(Modifier::BOLD))
                    }
                    else {
                        Span::styled("-", bar_style.add_modifier(Modifier::BOLD))
                    }
                }
                else if index < current_len {
                    Span::styled("=", bar_style.add_modifier(Modifier::BOLD))
                }
                else {
                    Span::styled("·", bar_off_style)
                }
            })
            .collect::<Vec<_>>();

        let limit_style = Style::default().fg(Color::White);
        buffer.set_string(area.x, area.y, "[", limit_style);
        buffer.set_spans(area.x + 1, area.y, &Spans::from(bar), 10);
        buffer.set_string(area.x + bar_len + 1, area.y, "]", limit_style);

        let numbers = Spans::from(vec![
            Span::styled(format!("{:>3}", self.current), bar_style.add_modifier(Modifier::BOLD)),
            Span::styled("/", limit_style),
            Span::styled(self.max.to_string(), bar_style.add_modifier(Modifier::BOLD)),
        ]);
        buffer.set_spans(area.x + bar_len + 3, area.y, &numbers, 7);
    }
}

#[derive(derive_new::new)]
struct MapWidget<'a> {
    state: &'a State,
    arena: &'a Arena,
}

impl MapWidget<'_> {
    pub fn dimension(map_size: u16) -> (u16, u16) {
        (map_size * 2 - 1, map_size)
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Player sight
        let user_player = &self.state.server.game.arena().user_player;
        let player = &self.state.server.game.players[user_player.player_id];
        if let Some(entity) = &self.state.server.game.arena().entities.get(&player.entity_id) {
            let pos = entity.position + user_player.direction.to_vec2();
            let x = pos.x as u16 * 2;
            let y = pos.y as u16;
            let style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD);
            buffer.set_string(area.x + x, area.y + y, &"·", style);
        }

        // Border
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .render(area, buffer);

        // Spells
        for (_, spell) in &self.state.server.game.arena().spells {
            let x = spell.position.x as u16 * 2;
            let y = spell.position.y as u16;
            let style = Style::default().fg(Color::Indexed(208)).remove_modifier(Modifier::BOLD);
            buffer.set_string(area.x + x, area.y + y, "o", style);
        }

        // Entities
        for (id, entity) in &self.state.server.game.arena().entities {
            let x = entity.position.x as u16 * 2;
            let y = entity.position.y as u16;
            let character = self.state.server.game.characters.get(&entity.character_id).unwrap();
            let color = match self.arena.damaged_entities.get(id) {
                Some(_) => Color::LightRed,
                None => Color::White,
            };
            let style = match character.id() {
                CharacterId::Player(_) => Style::default().fg(color).add_modifier(Modifier::BOLD),
                _ => Style::default().fg(color),
            };
            buffer.set_string(area.x + x, area.y + y, &character.symbol().to_string(), style);
        }

        FinishGameMessageWidget::new(self.state).render(area, buffer);
    }
}

#[derive(derive_new::new)]
struct FinishGameMessageWidget<'a> {
    state: &'a State,
}

impl Widget for FinishGameMessageWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if let GameStatus::Finished = self.state.server.game.status {
            let winner_points = self.state.server.game_info().winner_points;
            let winner_player =
                self.state.server.game.players.iter().find(|p| p.points >= winner_points).unwrap();
            let winner_character = &self.state.server.game.characters[&winner_player.character_id];

            let message = vec![
                Spans::from(vec![
                    Span::raw("Player "),
                    Span::styled(
                        winner_character.symbol().to_string(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" wins!"),
                ]),
                Spans::from(Span::raw("")),
                Spans::from(vec![
                    Span::raw("Press"),
                    Span::styled(
                        " <Enter> ",
                        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
                    ),
                    Span::raw("to back to the menu"),
                ]),
            ];

            let height = message.len() as u16;
            Paragraph::new(message)
                .alignment(Alignment::Center)
                .render(util::vertically_centered(area, height), buffer);
        }
    }
}

#[derive(derive_new::new)]
struct NotificationLabelWidget<'a> {
    state: &'a State,
}

impl NotificationLabelWidget<'_> {
    const HEIGHT: u16 = 2;
}

impl Widget for NotificationLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let messages = match self.state.server.game.next_arena_timestamp {
            Some(timestamp) => {
                let secs = timestamp.saturating_duration_since(Instant::now()).as_secs() + 1;
                let winner_arena_player =
                    self.state.server.game.players.iter().find(|p| {
                        self.state.server.game.arena().entities.contains_key(&p.entity_id)
                    });

                let player_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
                let style = Style::default().fg(Color::LightCyan);
                let winner_message = match winner_arena_player {
                    Some(player) => {
                        let character = &self.state.server.game.characters[&player.character_id];
                        Spans::from(vec![
                            Span::styled("Player ", style),
                            Span::styled(character.symbol().to_string(), player_style),
                            Span::styled(" survived", style),
                            Span::styled(". ", style),
                        ])
                    }
                    None => Spans::from(vec![Span::styled("No one survived", style)]),
                };

                vec![
                    winner_message,
                    Spans::from(vec![
                        Span::styled("Starting new arena in ", style),
                        Span::styled(secs.to_string(), style.add_modifier(Modifier::BOLD)),
                        Span::styled("...", style),
                    ]),
                ]
            }
            None => vec![],
        };

        Paragraph::new(messages).alignment(Alignment::Center).render(area, buffer);
    }
}
