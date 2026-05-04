use axum::http::StatusCode;
use std::sync::Arc;

use crate::domain::channel_events::IncomingEvent;

use super::server::{AppState, process_event};

pub async fn dispatch_event(state: &Arc<AppState>, event: IncomingEvent) -> StatusCode {
    match process_event(state, event).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, "Failed to handle event");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
