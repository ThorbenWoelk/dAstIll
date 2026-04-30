use super::{ActiveChatHandle, await_or_cancel, cancelled_error};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Notify;

#[tokio::test]
async fn await_or_cancel_returns_cancelled_while_future_is_pending() {
    let active_chat = ActiveChatHandle::new();
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let completed = Arc::new(AtomicBool::new(false));

    let result_task = tokio::spawn({
        let active_chat = active_chat.clone();
        let started = Arc::clone(&started);
        let release = Arc::clone(&release);
        let completed = Arc::clone(&completed);
        async move {
            await_or_cancel(&active_chat, async move {
                started.notify_one();
                release.notified().await;
                completed.store(true, Ordering::SeqCst);
                "finished"
            })
            .await
        }
    });

    started.notified().await;
    active_chat.cancel();

    let result = result_task.await.expect("pending future task should join");
    assert_eq!(result, Err(cancelled_error()));
    assert!(!completed.load(Ordering::SeqCst));

    release.notify_waiters();
}
