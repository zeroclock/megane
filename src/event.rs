pub enum LogGroupEvent {
    FetchLogGroups,
    Abort,
}

#[derive(PartialEq, Debug)]
pub enum LogEventEvent {
    FetchLogEvents(String, Option<String>),
    Abort,
}

#[derive(Debug, PartialEq)]
pub enum Event<I> {
    Input(I),
    Tick,
}
