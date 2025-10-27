# Ticket 3: Migrate File Watcher to Notify

**Task:** Migrate the file system watcher from `@parcel/watcher` and `chokidar` to `notify`.

**Description:** The application uses `@parcel/watcher` and `chokidar` to monitor file system changes. This ticket involves migrating this functionality to `notify`, a cross-platform file system notification library for Rust. The goal is to replicate the existing file watching and event handling logic.

**Acceptance Criteria:**
- The application can watch files and directories for changes.
- File system events (create, modify, delete) are correctly handled.
- The file watcher is integrated with the main application logic.
- The implementation is efficient and does not consume excessive system resources.

**Files to Migrate:**
- `packages/opencode/src/file/watcher.ts`

**Suggested Rust Crates:**
- `notify`
- `tokio` (for the async runtime)

**Unit Test Examples:**

```rust
// in tests/watcher.rs

#[cfg(test)]
mod tests {
    use super::*;
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::path::Path;
    use std::time::Duration;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_file_watcher() {
        let (tx, mut rx) = mpsc::channel(1);

        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
            tx.blocking_send(res).unwrap();
        })
        .unwrap();

        let path = Path::new("test_dir");
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        // Create a file to trigger an event
        let file_path = path.join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        let event = rx.recv().await.unwrap();
        assert!(event.is_ok());

        // Clean up
        std::fs::remove_file(&file_path).unwrap();
        std::fs::remove_dir(path).unwrap();
    }
}
```
