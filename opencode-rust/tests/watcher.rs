use opencode_rust::watcher;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[tokio::test]
async fn test_file_watcher() {
    let test_dir = Path::new("test_dir");
    if !test_dir.exists() {
        fs::create_dir(test_dir).unwrap();
    }

    let watch_task = tokio::spawn(async move {
        let _ = watcher::watch(test_dir).await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let file_path = test_dir.join("test.txt");
    fs::write(&file_path, "test").unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    fs::remove_file(&file_path).unwrap();
    fs::remove_dir(test_dir).unwrap();

    watch_task.abort();
}
