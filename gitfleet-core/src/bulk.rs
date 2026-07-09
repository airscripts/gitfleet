use std::future::Future;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[derive(Debug, Clone)]
pub struct BulkItemResult<T> {
    pub index: usize,
    pub item: T,
    pub success: bool,
    pub error: Option<String>,
}

pub async fn run_bulk<T, F, Fut>(
    items: &[T],
    worker: F,
    concurrency: usize,
) -> Vec<BulkItemResult<T>>
where
    T: Clone + Send + 'static,
    F: Fn(T, usize) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), String>> + Send,
{
    run_bulk_with_cancel(items, worker, concurrency, CancellationToken::new(), false).await
}

pub async fn run_bulk_with_cancel<T, F, Fut>(
    items: &[T],
    worker: F,
    concurrency: usize,
    cancel: CancellationToken,
    cancel_on_error: bool,
) -> Vec<BulkItemResult<T>>
where
    T: Clone + Send + 'static,
    F: Fn(T, usize) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), String>> + Send,
{
    let limit = concurrency.max(1);

    let results: Arc<tokio::sync::Mutex<Vec<Option<BulkItemResult<T>>>>> = Arc::new(
        tokio::sync::Mutex::new((0..items.len()).map(|_| None).collect()),
    );

    let next_index = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let worker = Arc::new(worker);
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let tracker = TaskTracker::new();

    for _ in 0..limit.min(items.len()) {
        let results = results.clone();

        let next_index = next_index.clone();
        let items_vec = items.to_vec();

        let worker = worker.clone();
        let cancel = cancel.clone();

        let cancelled = cancelled.clone();

        tracker.spawn(async move {
            loop {
                if cancel.is_cancelled() || cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                let idx = next_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if idx >= items_vec.len() {
                    break;
                }

                let item = items_vec[idx].clone();

                match worker(item.clone(), idx).await {
                    Ok(()) => {
                        let mut guard = results.lock().await;
                        guard[idx] = Some(BulkItemResult {
                            index: idx,
                            item,
                            success: true,
                            error: None,
                        });
                    }

                    Err(e) => {
                        let mut guard = results.lock().await;
                        guard[idx] = Some(BulkItemResult {
                            index: idx,
                            item,
                            success: false,
                            error: Some(e),
                        });

                        if cancel_on_error {
                            cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
                            cancel.cancel();
                        }
                    }
                }
            }
        });
    }

    tracker.close();
    tracker.wait().await;

    let guard = results.lock().await;
    guard.iter().flatten().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_bulk_all_success() {
        let items = vec!["a", "b", "c"];
        let results = run_bulk(
            &items,
            |item, _idx| {
                let item = item.to_string();
                async move {
                    if item.is_empty() {
                        Err("empty item".to_string())
                    } else {
                        Ok(())
                    }
                }
            },
            2,
        )
        .await;

        assert_eq!(results.len(), 3);

        for r in &results {
            assert!(r.success);

            assert!(r.error.is_none());
        }

        assert_eq!(results[0].item, "a");

        assert_eq!(results[1].item, "b");
        assert_eq!(results[2].item, "c");
    }

    #[tokio::test]
    async fn test_run_bulk_all_failure() {
        let items = vec![1, 2, 3];
        let results = run_bulk(
            &items,
            |_item, _idx| async { Err("always fails".to_string()) },
            2,
        )
        .await;

        assert_eq!(results.len(), 3);

        for r in &results {
            assert!(!r.success);

            assert_eq!(r.error.as_deref(), Some("always fails"));
        }
    }

    #[tokio::test]
    async fn test_run_bulk_mixed() {
        let items = vec![10, 20, 30];
        let results = run_bulk(
            &items,
            |item, _idx| async move {
                if item > 20 {
                    Err("too large".to_string())
                } else {
                    Ok(())
                }
            },
            2,
        )
        .await;

        assert_eq!(results.len(), 3);

        assert!(results[0].success);
        assert!(results[1].success);

        assert!(!results[2].success);
        assert_eq!(results[2].error.as_deref(), Some("too large"));
    }

    #[tokio::test]
    async fn test_run_bulk_empty() {
        let items: Vec<String> = vec![];
        let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 2).await;

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_run_bulk_single_item() {
        let items = vec!["only"];
        let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 4).await;

        assert_eq!(results.len(), 1);

        assert!(results[0].success);
        assert_eq!(results[0].item, "only");
    }

    #[tokio::test]
    async fn test_run_bulk_preserves_indices() {
        let items = vec!["x", "y", "z"];
        let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 1).await;

        assert_eq!(results[0].index, 0);

        assert_eq!(results[1].index, 1);
        assert_eq!(results[2].index, 2);
    }

    #[tokio::test]
    async fn test_run_bulk_concurrency_one() {
        let items = vec![1, 2, 3];
        let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 1).await;

        assert_eq!(results.len(), 3);

        assert!(results.iter().all(|r| r.success));
    }

    #[tokio::test]
    async fn test_run_bulk_high_concurrency() {
        let items: Vec<i32> = (0..20).collect();

        let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 10).await;

        assert_eq!(results.len(), 20);

        assert!(results.iter().all(|r| r.success));
    }

    #[tokio::test]
    async fn test_run_bulk_with_cancel_stops_early() {
        let cancel = CancellationToken::new();
        cancel.cancel();

        let items = vec![1, 2, 3, 4, 5];
        let results =
            run_bulk_with_cancel(&items, |_item, _idx| async { Ok(()) }, 2, cancel, false).await;

        assert!(results.iter().all(|r| r.index < items.len()));
    }

    #[tokio::test]
    async fn test_run_bulk_cancel_on_error() {
        let items = vec![1, 2, 3, 4, 5];
        let results = run_bulk_with_cancel(
            &items,
            |item, _idx| async move {
                if item == 3 {
                    Err("fail on 3".to_string())
                } else {
                    Ok(())
                }
            },
            1,
            CancellationToken::new(),
            true,
        )
        .await;

        assert!(results.iter().any(|r| !r.success));
    }

    #[tokio::test]
    async fn test_run_bulk_no_cancel_on_error() {
        let items = vec![1, 2, 3, 4, 5];
        let results = run_bulk_with_cancel(
            &items,
            |item, _idx| async move {
                if item == 3 {
                    Err("fail on 3".to_string())
                } else {
                    Ok(())
                }
            },
            1,
            CancellationToken::new(),
            false,
        )
        .await;

        assert_eq!(results.len(), 5);

        assert!(results[0].success);
        assert!(results[1].success);

        assert!(!results[2].success);
        assert!(results[3].success);

        assert!(results[4].success);
    }
}
