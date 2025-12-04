// File: rusty-sync/src/sse.rs
// Purpose: Server-Sent Events for real-time sync updates

use axum::{
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Extension,
};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::change_tracker::ChangeLog;

/// SSE handler for sync events
pub async fn sync_events_handler(
    Extension(rx): Extension<Arc<broadcast::Sender<ChangeLog>>>,
) -> impl IntoResponse {
    let stream = BroadcastStream::new(rx.subscribe());

    let event_stream = stream
        .filter_map(|result| async move {
            match result {
                Ok(change) => {
                    // Convert ChangeLog to SSE event
                    let json = serde_json::to_string(&change).ok()?;
                    Some(Ok::<_, Infallible>(
                        Event::default().data(json).event("sync"),
                    ))
                }
                Err(_) => None,
            }
        });

    Sse::new(event_stream).keep_alive(KeepAlive::default())
}

/// Create SSE stream from broadcast receiver
pub fn create_sse_stream(
    rx: broadcast::Receiver<ChangeLog>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    BroadcastStream::new(rx).filter_map(|result| async move {
        match result {
            Ok(change) => {
                let json = serde_json::to_string(&change).ok()?;
                Some(Ok(Event::default().data(json).event("sync")))
            }
            Err(_) => None,
        }
    })
}
