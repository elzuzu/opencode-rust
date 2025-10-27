use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::event::{Event, EventKind, ModifyKind, RenameMode};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, error};

use crate::util::config::Info;

const IGNORED_FOLDERS: &[&str] = &[
    "node_modules",
    "bower_components",
    ".pnpm-store",
    "vendor",
    "dist",
    "build",
    "out",
    ".next",
    "target",
    "bin",
    "obj",
    ".git",
    ".svn",
    ".hg",
    ".vscode",
    ".idea",
    ".turbo",
    ".output",
    "desktop",
    ".sst",
];

const IGNORED_GLOBS: &[&str] = &[
    "**/*.swp",
    "**/*.swo",
    "**/.DS_Store",
    "**/Thumbs.db",
    "**/logs/**",
    "**/tmp/**",
    "**/temp/**",
    "**/*.log",
    "**/coverage/**",
    "**/.nyc_output/**",
];

#[derive(Debug, Clone, Default)]
pub struct WatchOptions {
    pub ignore: Vec<String>,
}

impl WatchOptions {
    pub fn with_ignore(ignore: Vec<String>) -> Self {
        Self { ignore }
    }
}

impl From<&Info> for WatchOptions {
    fn from(info: &Info) -> Self {
        Self {
            ignore: info.watcher_ignore_patterns(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEvent {
    pub path: PathBuf,
    pub kind: FileEventKind,
}

pub struct FileWatcher {
    _watcher: Option<RecommendedWatcher>,
    processor: Option<JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    events: mpsc::Receiver<FileEvent>,
}

impl FileWatcher {
    pub async fn next(&mut self) -> Option<FileEvent> {
        self.events.recv().await
    }

    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.processor.take() {
            let _ = handle.await;
        }
    }
}

pub async fn watch(path: &Path, options: WatchOptions) -> Result<FileWatcher> {
    let canonical_root = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let matcher = IgnoreMatcher::new(canonical_root.clone(), &options.ignore);
    let (raw_tx, raw_rx) = mpsc::channel::<notify::Result<Event>>(256);
    let (event_tx, event_rx) = mpsc::channel(256);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let mut watcher = notify::recommended_watcher({
        move |res| {
            if let Err(err) = raw_tx.blocking_send(res) {
                error!("failed to push watch event: {err}");
            }
        }
    })?;

    watcher.configure(
        NotifyConfig::default()
            .with_poll_interval(Duration::from_millis(250))
            .with_compare_contents(true),
    )?;

    watcher
        .watch(&canonical_root, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", canonical_root.display()))?;

    let processor = tokio::spawn(async move {
        process_events(raw_rx, shutdown_rx, event_tx, matcher).await;
    });

    Ok(FileWatcher {
        _watcher: Some(watcher),
        processor: Some(processor),
        shutdown: Some(shutdown_tx),
        events: event_rx,
    })
}

struct IgnoreMatcher {
    root: PathBuf,
    folders: HashSet<String>,
    patterns: Vec<String>,
}

impl IgnoreMatcher {
    fn new(root: PathBuf, extra: &[String]) -> Self {
        let mut patterns = IGNORED_GLOBS
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>();
        patterns.extend(extra.iter().cloned());
        let folders = IGNORED_FOLDERS
            .iter()
            .map(|name| name.to_string())
            .collect();
        Self {
            root,
            folders,
            patterns,
        }
    }

    fn matches(&self, path: &Path) -> bool {
        let relative = path
            .strip_prefix(&self.root)
            .ok()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(path);

        for component in relative.iter() {
            if let Some(name) = component.to_str() {
                if self.folders.contains(name) {
                    return true;
                }
            }
        }
        if let Some(path_str) = normalize_path(relative) {
            for pattern in &self.patterns {
                if glob_matches(pattern, &path_str) {
                    return true;
                }
            }
        }
        false
    }
}

async fn process_events(
    mut raw_rx: mpsc::Receiver<notify::Result<Event>>,
    mut shutdown: oneshot::Receiver<()>,
    event_tx: mpsc::Sender<FileEvent>,
    matcher: IgnoreMatcher,
) {
    loop {
        tokio::select! {
            _ = &mut shutdown => {
                break;
            }
            maybe_event = raw_rx.recv() => {
                let Some(event) = maybe_event else {
                    break;
                };
                match event {
                    Ok(event) => {
                        if let Some(mapped) = map_event(event, &matcher) {
                            if event_tx.send(mapped).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        error!("watch error: {err}");
                    }
                }
            }
        }
    }
}

fn map_event(event: Event, matcher: &IgnoreMatcher) -> Option<FileEvent> {
    let path = select_path(&event, matcher)?;
    let mut kind = match event.kind {
        EventKind::Create(_) => FileEventKind::Created,
        EventKind::Modify(ModifyKind::Name(mode)) => match mode {
            RenameMode::From => FileEventKind::Deleted,
            RenameMode::To => FileEventKind::Created,
            RenameMode::Both | RenameMode::Any | RenameMode::Other => FileEventKind::Modified,
        },
        EventKind::Modify(ModifyKind::Data(_))
        | EventKind::Modify(ModifyKind::Metadata(_))
        | EventKind::Modify(ModifyKind::Other)
        | EventKind::Modify(ModifyKind::Any) => FileEventKind::Modified,
        EventKind::Remove(_) => FileEventKind::Deleted,
        EventKind::Access(_) | EventKind::Other | EventKind::Any => return None,
    };
    if matches!(kind, FileEventKind::Modified) && !path.exists() {
        kind = FileEventKind::Deleted;
    }
    debug!(?path, ?kind, "file event");
    Some(FileEvent { path, kind })
}

fn select_path(event: &Event, matcher: &IgnoreMatcher) -> Option<PathBuf> {
    for path in event.paths.iter().rev() {
        if !matcher.matches(path) {
            return Some(path.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tokio::time::{sleep, timeout};
    use uuid::Uuid;

    #[tokio::test]
    async fn emits_events_for_changes() {
        let path = temp_dir();
        let mut watcher = watch(path.as_path(), WatchOptions::default())
            .await
            .unwrap();

        let file = path.join("test.txt");
        fs::write(&file, "hello").unwrap();

        let event = timeout(Duration::from_secs(5), watcher.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.kind, FileEventKind::Created);
        assert_eq!(event.path, file);

        fs::write(&file, "world").unwrap();
        let event = timeout(Duration::from_secs(5), watcher.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.kind, FileEventKind::Modified);

        fs::remove_file(&file).unwrap();
        let event = timeout(Duration::from_secs(5), watcher.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.kind, FileEventKind::Deleted);

        watcher.shutdown().await;
        let _ = fs::remove_dir_all(&path);
    }

    #[tokio::test]
    async fn applies_ignore_patterns() {
        let path = temp_dir();
        let mut options = WatchOptions::default();
        options.ignore.push("**/*.log".to_string());
        let mut watcher = watch(path.as_path(), options).await.unwrap();

        let ignored_dir = path.join("node_modules");
        fs::create_dir_all(&ignored_dir).unwrap();
        let ignored_file = ignored_dir.join("ignored.txt");
        fs::write(&ignored_file, "ignored").unwrap();

        let log_file = path.join("debug.log");
        fs::write(&log_file, "ignored").unwrap();

        sleep(Duration::from_millis(200)).await;
        let maybe_event = timeout(Duration::from_millis(500), watcher.next()).await;
        assert!(
            maybe_event.is_err(),
            "no events should be emitted for ignored files"
        );

        let tracked = path.join("tracked.txt");
        fs::write(&tracked, "data").unwrap();
        let event = timeout(Duration::from_secs(5), watcher.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.path, tracked);

        watcher.shutdown().await;
        let _ = fs::remove_dir_all(&path);
    }

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("opencode-watcher-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}

fn normalize_path(path: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in path.iter() {
        let segment = component.to_str()?;
        parts.push(segment);
    }
    Some(parts.join("/").replace("\\", "/"))
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let normalized_pattern = pattern.replace('\\', "/");
    let mut pattern_parts = Vec::new();
    for part in normalized_pattern.split('/') {
        if part.is_empty() {
            continue;
        }
        pattern_parts.push(part);
    }
    let path_parts: Vec<&str> = candidate.split('/').collect();
    glob_match_segments(&pattern_parts, &path_parts, 0, 0)
}

fn glob_match_segments(pattern: &[&str], path: &[&str], mut pi: usize, mut ti: usize) -> bool {
    while pi < pattern.len() {
        let segment = pattern[pi];
        if segment == "**" {
            pi += 1;
            if pi == pattern.len() {
                return true;
            }
            while ti <= path.len() {
                if glob_match_segments(pattern, path, pi, ti) {
                    return true;
                }
                if ti == path.len() {
                    break;
                }
                ti += 1;
            }
            return false;
        }
        if ti == path.len() {
            return false;
        }
        if !segment_matches(segment, path[ti]) {
            return false;
        }
        pi += 1;
        ti += 1;
    }
    ti == path.len()
}

fn segment_matches(pattern: &str, text: &str) -> bool {
    let pat = pattern.as_bytes();
    let txt = text.as_bytes();
    let mut pi = 0;
    let mut ti = 0;
    let mut star = None;
    let mut star_ti = 0;

    while ti < txt.len() {
        if pi < pat.len() && (pat[pi] == b'?' || pat[pi] == txt[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < pat.len() && pat[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            star_ti = ti;
        } else if let Some(star_idx) = star {
            pi = star_idx + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < pat.len() && pat[pi] == b'*' {
        pi += 1;
    }

    pi == pat.len()
}
