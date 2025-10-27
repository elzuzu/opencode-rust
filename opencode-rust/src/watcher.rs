use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;

pub async fn watch(path: &Path) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel(1);

    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
        tx.blocking_send(res).unwrap();
    })?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => println!("event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
