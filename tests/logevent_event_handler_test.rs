use std::sync::{Arc, Mutex};

use megane::{
    client::LogClient,
    event::{LogEventEvent, TailLogEventEvent},
    handler::{logevent_event_handler::LogEventEventHandler, EventHandler},
    state::{
        logevents_state::LogEventsState,
        search_state::{SearchMode, SearchState},
    },
};

mod common;

#[tokio::test]
async fn test_run_basis() {
    let state = Arc::new(Mutex::new(LogEventsState::new()));
    let (mut inst_tx, inst_rx) = tokio::sync::mpsc::channel::<LogEventEvent>(1);
    let (tail_inst_tx, _tail_inst_rx) = tokio::sync::mpsc::channel::<TailLogEventEvent>(1);
    let mock_client = common::get_mock_client("logevents_01.json");
    let mut handler = LogEventEventHandler::new(
        LogClient::new(mock_client),
        Arc::clone(&state),
        inst_rx,
        tail_inst_tx,
    );
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let search_state = Some(SearchState::new(String::default(), SearchMode::TwelveHours));
    assert!(inst_tx
        .send(LogEventEvent::FetchLogEvents(
            "log group name".to_string(),
            None,
            search_state,
            true
        ))
        .await
        .is_ok());
    assert!(inst_tx.send(LogEventEvent::Abort).await.is_ok());

    let _ = handle.await.unwrap();

    for i in 0..=4 {
        assert_eq!(
            Some(format!("log_event_{}", (i + 1).to_string())),
            state.lock().unwrap().events.items().get(i).unwrap().message
        );
        assert_eq!(
            Some((i + 1).to_string()),
            state
                .lock()
                .unwrap()
                .events
                .items()
                .get(i)
                .unwrap()
                .event_id
        );
    }
}

#[tokio::test]
async fn test_run_send_tail() {
    let state = Arc::new(Mutex::new(LogEventsState::new()));
    let (mut inst_tx, inst_rx) = tokio::sync::mpsc::channel::<LogEventEvent>(1);
    let (tail_inst_tx, mut tail_inst_rx) = tokio::sync::mpsc::channel::<TailLogEventEvent>(1);
    let mock_client = common::get_mock_client("logevents_01.json");
    let mut handler = LogEventEventHandler::new(
        LogClient::new(mock_client),
        Arc::clone(&state),
        inst_rx,
        tail_inst_tx,
    );
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let search_state = Some(SearchState::new(String::default(), SearchMode::Tail));
    let search_state_clone = search_state.clone();

    let assert_handle = tokio::spawn(async move {
        let event = tail_inst_rx.recv().await.unwrap();
        assert_eq!(
            TailLogEventEvent::Start("log group name".to_string(), None, search_state_clone, true),
            event
        );
    });

    assert!(inst_tx
        .send(LogEventEvent::FetchLogEvents(
            "log group name".to_string(),
            None,
            search_state,
            true
        ))
        .await
        .is_ok());
    assert!(inst_tx.send(LogEventEvent::Abort).await.is_ok());

    let _ = handle.await.unwrap();
    let _ = assert_handle.await.unwrap();
}
