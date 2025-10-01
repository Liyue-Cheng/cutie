/// SSE 端点与订阅管理
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
};
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _, Stream};

use super::models::DomainEvent;
use crate::startup::AppState;

/// SSE 状态（用于事件广播）
pub struct SseState {
    /// 事件广播通道
    tx: broadcast::Sender<DomainEvent>,
}

impl SseState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    /// 广播事件到所有订阅者
    pub fn broadcast(&self, event: DomainEvent) {
        let _ = self.tx.send(event);
    }

    /// 创建新订阅者
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.tx.subscribe()
    }
}

impl Default for SseState {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE 端点处理器
///
/// GET /api/events/stream
pub async fn handle(
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = app_state.sse_state().subscribe();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| match result {
        Ok(event) => {
            let sse_event = Event::default()
                .event(&event.event_type)
                .data(event.to_sse_data())
                .id(event.event_id.to_string());
            Some(Ok(sse_event))
        }
        Err(_) => None,
    });

    Sse::new(event_stream).keep_alive(KeepAlive::default())
}
