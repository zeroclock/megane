use crate::state::search_state::SearchState;

pub enum LogGroupEvent {
    FetchLogGroups,
    Abort,
}

#[derive(PartialEq, Debug)]
pub enum LogEventEvent {
    // log_group_name, next_token, search_conditions, need_reset
    FetchLogEvents(String, Option<String>, Option<SearchState>, bool),
    Abort,
}

#[derive(Debug, PartialEq)]
pub enum Event<I> {
    Input(I),
    Tick,
}
