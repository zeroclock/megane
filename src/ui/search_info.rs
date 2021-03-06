use std::marker::PhantomData;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{state::search_state::*, ui::Drawable};

pub struct SearchInfo<B>
where
    B: Backend,
{
    state: SearchState,
    _phantom: PhantomData<B>,
}

impl<B> SearchInfo<B>
where
    B: Backend,
{
    pub fn new(state: SearchState) -> Self {
        SearchInfo {
            state,
            _phantom: PhantomData,
        }
    }

    fn get_msg(&self) -> String {
        format!("query: [{}], mode: [{}]", self.state.query, self.state.mode)
    }

    pub fn set_state(&mut self, new_state: SearchState) {
        self.state = new_state;
    }

    pub fn get_state(&self) -> SearchState {
        self.state.clone()
    }

    pub fn is_same_state(&self, other_state: &SearchState) -> bool {
        &self.state == other_state
    }
}

impl<B> Default for SearchInfo<B>
where
    B: Backend,
{
    fn default() -> Self {
        SearchInfo {
            state: SearchState::default(),
            _phantom: PhantomData,
        }
    }
}

impl<B> Clone for SearchInfo<B>
where
    B: Backend,
{
    fn clone(&self) -> Self {
        SearchInfo::new(self.get_state())
    }
}

#[async_trait]
impl<B> Drawable<B> for SearchInfo<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        let block = Block::default().borders(Borders::NONE);
        let paragraph = Paragraph::new(self.get_msg())
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    async fn handle_event(&mut self, _event: KeyEvent) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use tui::backend::TestBackend;

    use super::*;
    use crate::state::search_state::{SearchMode, SearchState};

    #[test]
    fn test_set_state() {
        let mut search_info: SearchInfo<TestBackend> = SearchInfo::default();
        let query = String::from("this is test query");
        let mode = SearchMode::ThirtyMinutes;
        let state = SearchState::new(query.clone(), mode.clone());
        assert!(!search_info.is_same_state(&state));
        search_info.set_state(state.clone());
        assert!(search_info.is_same_state(&state));
    }
}
