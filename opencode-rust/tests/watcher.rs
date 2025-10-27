use opencode_rust::util::config;
use opencode_rust::watcher::{self, FileEventKind, WatchOptions};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

#[tokio::test]
async fn file_watcher_reports_lifecycle_events() {
    let path = temp_dir();
    let mut handle = watcher::watch(path.as_path(), WatchOptions::default())
        .await
        .unwrap();

    let file = path.join("watch.txt");
    fs::write(&file, "created").unwrap();
    let event = timeout(Duration::from_secs(5), handle.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.kind, FileEventKind::Created);

    fs::write(&file, "updated").unwrap();
    let event = timeout(Duration::from_secs(5), handle.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.kind, FileEventKind::Modified);

    fs::remove_file(&file).unwrap();
    let event = timeout(Duration::from_secs(5), handle.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.kind, FileEventKind::Deleted);

    handle.shutdown().await;
    let _ = std::fs::remove_dir_all(&path);
}

#[tokio::test]
async fn file_watcher_honors_configured_ignores() {
    let path = temp_dir();
    let mut options = WatchOptions::default();
    options.ignore.push("**/*.tmp".into());
    let mut handle = watcher::watch(path.as_path(), options).await.unwrap();

    let ignored = path.join("ignored.tmp");
    fs::write(&ignored, "ignored").unwrap();

    assert!(
        timeout(Duration::from_millis(500), handle.next())
            .await
            .is_err()
    );

    let tracked = path.join("tracked.txt");
    fs::write(&tracked, "tracked").unwrap();
    let event = timeout(Duration::from_secs(5), handle.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.path, tracked);

    handle.shutdown().await;
    let _ = std::fs::remove_dir_all(&path);
}

#[tokio::test]
async fn file_watcher_uses_config_ignore_patterns() {
    let path = temp_dir();
    let info = config::parse_info(r#"{"watcher": {"ignore": ["**/*.tmp"]}}"#).unwrap();
    let mut handle = watcher::watch(path.as_path(), (&info).into())
        .await
        .unwrap();

    let ignored = path.join("ignored.tmp");
    fs::write(&ignored, "ignored").unwrap();

    assert!(
        timeout(Duration::from_millis(500), handle.next())
            .await
            .is_err()
    );

    let tracked = path.join("tracked.txt");
    fs::write(&tracked, "tracked").unwrap();
    let event = timeout(Duration::from_secs(5), handle.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.path, tracked);

    handle.shutdown().await;
    let _ = std::fs::remove_dir_all(&path);
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-watcher-it-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
