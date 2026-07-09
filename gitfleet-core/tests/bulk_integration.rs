use gitfleet_core::bulk::{run_bulk, run_bulk_with_cancel};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_bulk_all_success() {
    let items = vec!["alpha", "beta", "gamma"];
    let results = run_bulk(&items, |_item, _idx| async move { Ok(()) }, 2).await;

    assert_eq!(results.len(), 3);

    assert!(results.iter().all(|r| r.success));
    assert!(results.iter().all(|r| r.error.is_none()));
}

#[tokio::test]
async fn test_bulk_preserves_order() {
    let items: Vec<i32> = (1..=10).collect();

    let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 3).await;

    let mut sorted: Vec<usize> = results.iter().map(|r| r.index).collect();
    sorted.sort();
    assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[tokio::test]
async fn test_bulk_mixed_results() {
    let items = vec![10, 20, 30, 40, 50];
    let results = run_bulk(
        &items,
        |item, _idx| async move {
            if item > 25 {
                Err("too large".to_string())
            } else {
                Ok(())
            }
        },
        2,
    )
    .await;

    assert_eq!(results.len(), 5);

    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.iter().filter(|r| !r.success).count();

    assert!(success_count > 0);

    assert!(fail_count > 0);
}

#[tokio::test]
async fn test_bulk_empty_input() {
    let items: Vec<String> = vec![];
    let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 2).await;

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_bulk_single_item() {
    let items = vec!["only"];
    let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 4).await;

    assert_eq!(results.len(), 1);

    assert!(results[0].success);
    assert_eq!(results[0].item, "only");
}

#[tokio::test]
async fn test_bulk_cancel_before_start() {
    let cancel = CancellationToken::new();
    cancel.cancel();

    let items = vec![1, 2, 3, 4, 5];
    let results =
        run_bulk_with_cancel(&items, |_item, _idx| async { Ok(()) }, 2, cancel, false).await;

    assert!(results.iter().all(|r| r.index < items.len()));
}

#[tokio::test]
async fn test_bulk_cancel_on_error() {
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
async fn test_bulk_no_cancel_on_error() {
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

#[tokio::test]
async fn test_bulk_all_failures() {
    let items = vec![1, 2, 3];
    let results = run_bulk(
        &items,
        |_item, _idx| async { Err("always fails".to_string()) },
        2,
    )
    .await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| !r.success));

    assert!(results.iter().all(|r| r.error.is_some()));
}

#[tokio::test]
async fn test_bulk_high_concurrency() {
    let items: Vec<i32> = (0..100).collect();

    let results = run_bulk(&items, |_item, _idx| async { Ok(()) }, 10).await;

    assert_eq!(results.len(), 100);

    assert!(results.iter().all(|r| r.success));
}
