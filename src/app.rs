use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::ui::{event_area::EventArea, side_menu::SideMenu, Drawable};

/// which component selected
enum SelectState {
    SideMenu,
    EventAreas(usize),
}

pub struct App<B>
where
    B: Backend,
{
    side_menu: SideMenu<B>,
    event_areas: Vec<EventArea<B>>,
    select_state: SelectState,
    fold: bool,
}

impl<B> App<B>
where
    B: Backend,
{
    pub async fn new(side_menu: SideMenu<B>, event_areas: Vec<EventArea<B>>, fold: bool) -> Self {
        App {
            side_menu,
            event_areas,
            select_state: SelectState::SideMenu,
            fold,
        }
    }

    pub fn split_event_area(&self, rect: Rect) -> Vec<Rect> {
        let constaints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let base_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constaints.as_ref())
            .split(rect);
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constaints.as_ref())
            .split(base_chunks[0]);
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constaints.as_ref())
            .split(base_chunks[1]);

        match self.event_areas.len() {
            1 => {
                vec![left_chunks[0]
                    .union(left_chunks[1])
                    .union(right_chunks[0])
                    .union(right_chunks[1])]
            }
            2 => {
                vec![
                    left_chunks[0].union(left_chunks[1]),
                    right_chunks[0].union(right_chunks[1]),
                ]
            }
            3 => {
                vec![
                    left_chunks[0],
                    right_chunks[0].union(right_chunks[1]),
                    left_chunks[1],
                ]
            }
            4 => {
                vec![
                    left_chunks[0],
                    right_chunks[0],
                    left_chunks[1],
                    right_chunks[1],
                ]
            }
            _ => vec![],
        }
    }

    pub fn toggle_side_fold(&mut self) {
        self.fold = !self.fold;
    }
}

impl<B> Default for App<B>
where
    B: Backend,
{
    fn default() -> Self {
        App {
            side_menu: SideMenu::default(),
            event_areas: vec![],
            select_state: SelectState::SideMenu,
            fold: false,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for App<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<B>, _area: Rect) {
        let (left, right) = if self.fold { (3, 97) } else { (30, 70) };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(left), Constraint::Percentage(right)].as_ref())
            .split(f.size());
        // update select state
        self.side_menu.set_select(match self.select_state {
            SelectState::SideMenu => true,
            SelectState::EventAreas(_) => false,
        });
        for (i, v) in self.event_areas.iter_mut().enumerate() {
            v.set_select(match self.select_state {
                SelectState::SideMenu => false,
                SelectState::EventAreas(idx) => {
                    if i == idx {
                        true
                    } else {
                        false
                    }
                }
            })
        }
        // draw side menu and event areas
        self.side_menu.draw(f, chunks[0]);
        let event_area_rects = self.split_event_area(chunks[1]);
        for (i, v) in self.event_areas.iter_mut().enumerate() {
            v.draw(f, event_area_rects[i]);
        }
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        let solved = match self.select_state {
            SelectState::SideMenu => self.side_menu.handle_event(event).await,
            SelectState::EventAreas(idx) => {
                if let Some(event_area) = self.event_areas.get_mut(idx) {
                    event_area.handle_event(event).await
                } else {
                    false
                }
            }
        };
        if !solved {
            match event.code {
                KeyCode::Tab => {
                    self.toggle_side_fold();
                }
                _ => {}
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};

    use super::*;
    use crate::test_helper::get_test_terminal;

    fn test_case(
        app: &mut App<TestBackend>,
        side_menu_color: Color,
        event_area_color: Color,
        lines: Vec<&str>,
        side_menu_length: u16,
    ) {
        let mut terminal = get_test_terminal(100, 10);
        let lines = if lines.len() > 0 {
            lines
        } else {
            vec![
                "┌Log Groups──────────────────┐                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "│                            │                                                                      ",
                "└────────────────────────────┘                                                                      ",
            ]
        };
        let mut expected = Buffer::with_lines(lines);
        for y in 0..10 {
            for x in 0..100 {
                let ch = expected.get_mut(x, y);
                if y == 0 || y == 9 {
                    if x >= side_menu_length {
                        if ch.symbol != " " {
                            ch.set_fg(event_area_color);
                        }
                    } else {
                        ch.set_fg(side_menu_color);
                    }
                } else {
                    if ch.symbol != " " {
                        if x >= side_menu_length {
                            ch.set_fg(event_area_color);
                        } else {
                            ch.set_fg(side_menu_color);
                        }
                    }
                }
            }
        }
        terminal
            .draw(|f| {
                app.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_split_event_area() {
        let mut app: App<TestBackend> = App::default();
        // no event areas
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect: Vec<Rect> = vec![];
        assert_eq!(expect, result);
        // 1 event area
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![Rect::new(0, 0, 100, 100)];
        assert_eq!(expect, result);
        // 2 event areas
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![Rect::new(0, 0, 50, 100), Rect::new(50, 0, 50, 100)];
        assert_eq!(expect, result);
        // 3 event areas
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![
            Rect::new(0, 0, 50, 50),
            Rect::new(50, 0, 50, 100),
            Rect::new(0, 50, 50, 50),
        ];
        assert_eq!(expect, result);
        // 4 event areas
        app.event_areas.push(EventArea::default());
        let result = app.split_event_area(Rect::new(0, 0, 100, 100));
        let expect = vec![
            Rect::new(0, 0, 50, 50),
            Rect::new(50, 0, 50, 50),
            Rect::new(0, 50, 50, 50),
            Rect::new(50, 50, 50, 50),
        ];
        assert_eq!(expect, result);
    }

    #[tokio::test]
    async fn test_draw() {
        let mut app: App<TestBackend> = App::default();
        test_case(&mut app, Color::Yellow, Color::White, vec![], 30);
        app.event_areas.push(EventArea::default());
        let lines = vec![
            "┌Log Groups──────────────────┐┌Events──────────────────────────────────────────────────────────────┐",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "└────────────────────────────┘└────────────────────────────────────────────────────────────────────┘",
        ];
        test_case(&mut app, Color::Yellow, Color::White, lines, 30);
        // folding side menu
        app.toggle_side_fold();
        let lines = vec![
            "┌L┐┌Events─────────────────────────────────────────────────────────────────────────────────────────┐",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "│ ││                                                                                               │",
            "└─┘└───────────────────────────────────────────────────────────────────────────────────────────────┘",
        ];
        test_case(&mut app, Color::Yellow, Color::White, lines, 3);
        // event area selected
        app.toggle_side_fold();
        app.select_state = SelectState::EventAreas(0);
        let lines = vec![
            "┌Log Groups──────────────────┐┌Events──────────────────────────────────────────────────────────────┐",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "│                            ││                                                                    │",
            "└────────────────────────────┘└────────────────────────────────────────────────────────────────────┘",
        ];
        test_case(&mut app, Color::White, Color::Yellow, lines, 30);
    }
}
