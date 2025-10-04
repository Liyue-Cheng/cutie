/// 事件分发器 - 从 outbox 扫描并推送到 SSE
use std::{sync::Arc, time::Duration};
use tokio::{time, sync::Semaphore};

use super::{outbox::EventOutboxRepository, SseState};

/// 事件分发器
///
/// 后台任务，定期扫描 event_outbox 表，将未分发的事件推送到 SSE 流
pub struct EventDispatcher {
    outbox_repo: Arc<dyn EventOutboxRepository>,
    sse_state: Arc<SseState>,
    interval_ms: u64,
    /// 写入串行化信号量（与应用其他写操作共享）
    write_semaphore: Arc<Semaphore>,
}

impl EventDispatcher {
    pub fn new(
        outbox_repo: Arc<dyn EventOutboxRepository>,
        sse_state: Arc<SseState>,
        interval_ms: u64,
        write_semaphore: Arc<Semaphore>,
    ) -> Self {
        Self {
            outbox_repo,
            sse_state,
            interval_ms,
            write_semaphore,
        }
    }

    /// 启动分发循环
    pub async fn start(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_millis(self.interval_ms));

        loop {
            interval.tick().await;

            if let Err(e) = self.dispatch_batch().await {
                eprintln!("[EventDispatcher] Error dispatching batch: {}", e);
            }
        }
    }

    /// 分发一批事件
    async fn dispatch_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 查询未分发事件
        let events = self.outbox_repo.fetch_undispatched(50).await?;

        if events.is_empty() {
            return Ok(());
        }

        // 广播并标记为已分发
        for (outbox_id, event) in events {
            self.sse_state.broadcast(event);
            
            // ✅ 获取写入许可，确保 mark_dispatched 与其他写操作串行
            let _permit = self.write_semaphore.acquire().await?;
            self.outbox_repo.mark_dispatched(outbox_id).await?;
            // _permit 自动 drop，释放许可
        }

        Ok(())
    }
}
