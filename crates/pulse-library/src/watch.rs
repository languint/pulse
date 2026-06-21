use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};

use crate::error::LibraryError;

pub struct LibraryWatcher {
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

impl LibraryWatcher {
    pub fn new(
        roots: &[PathBuf],
        debounce: Duration,
        on_change: impl FnMut() + Send + 'static,
    ) -> Result<Self, LibraryError> {
        let callback = Arc::new(Mutex::new(on_change));
        let mut debouncer =
            new_debouncer(debounce, move |result: DebounceEventResult| match result {
                Ok(events) if !events.is_empty() => {
                    tracing::debug!("library paths changed: count={}", events.len());
                    if let Ok(mut handler) = callback.lock() {
                        handler();
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("library watcher error: {e}");
                }
            })
            .map_err(LibraryError::Notify)?;

        for root in roots {
            debouncer
                .watcher()
                .watch(root, RecursiveMode::Recursive)
                .map_err(|source| LibraryError::Watch {
                    path: root.clone(),
                    source,
                })?;
        }

        Ok(Self {
            _debouncer: debouncer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
        thread,
        time::Duration as StdDuration,
    };

    #[test]
    fn watcher_notifies_on_file_change() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().to_path_buf();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_for_cb = Arc::clone(&hits);

        let _watcher = LibraryWatcher::new(
            std::slice::from_ref(&root),
            Duration::from_millis(100),
            move || {
                hits_for_cb.fetch_add(1, Ordering::SeqCst);
            },
        )?;

        let file = root.join("note.txt");
        fs::write(&file, b"1")?;
        fs::write(&file, b"2")?;

        let deadline = std::time::Instant::now() + StdDuration::from_secs(3);
        while std::time::Instant::now() < deadline {
            if hits.load(Ordering::SeqCst) > 0 {
                return Ok(());
            }
            thread::sleep(StdDuration::from_millis(50));
        }

        Err("watcher did not fire".into())
    }
}
