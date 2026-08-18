use crate::preview::{PreviewError, PreviewImage, PreviewLimits, decode_thumbnail};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::SystemTime;

const MAX_CACHE_ENTRIES: usize = 128;
const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub(in crate::desktop) enum PreviewEvent {
    Ready {
        generation: u64,
        preview: PreviewImage,
    },
    Failed {
        generation: u64,
        error: PreviewError,
    },
}

struct PreviewRequest {
    generation: u64,
    path: PathBuf,
    limits: PreviewLimits,
    cancel: Arc<AtomicBool>,
    callback: Box<dyn Fn(PreviewEvent) + Send>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

#[derive(Default)]
struct PreviewCache {
    entries: HashMap<CacheKey, PreviewImage>,
    order: VecDeque<CacheKey>,
    total_bytes: usize,
}

impl PreviewCache {
    fn get(&mut self, path: &Path) -> Option<PreviewImage> {
        let key = cache_key(path)?;
        let preview = self.entries.get(&key)?.clone();
        self.touch(&key);
        Some(preview)
    }

    fn insert(&mut self, path: &Path, preview: PreviewImage) {
        let Some(key) = cache_key(path) else {
            return;
        };
        let bytes = preview.rgba.len();
        if bytes > MAX_CACHE_BYTES {
            return;
        }
        if let Some(previous) = self.entries.insert(key.clone(), preview) {
            self.total_bytes = self.total_bytes.saturating_sub(previous.rgba.len());
        }
        self.total_bytes = self.total_bytes.saturating_add(bytes);
        self.touch(&key);
        while self.entries.len() > MAX_CACHE_ENTRIES || self.total_bytes > MAX_CACHE_BYTES {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(previous) = self.entries.remove(&oldest) {
                self.total_bytes = self.total_bytes.saturating_sub(previous.rgba.len());
            }
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        self.order.retain(|candidate| candidate != key);
        self.order.push_back(key.clone());
    }
}

fn cache_key(path: &Path) -> Option<CacheKey> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(CacheKey {
        path: path.to_path_buf(),
        size: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

pub(in crate::desktop) struct PreviewScheduler {
    pending: Arc<(Mutex<Option<PreviewRequest>>, Condvar)>,
    stop: Arc<AtomicBool>,
    active_cancel: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    generation: Arc<AtomicU64>,
}

impl PreviewScheduler {
    pub(in crate::desktop) fn new() -> Result<Self, ()> {
        let pending = Arc::new((Mutex::new(None::<PreviewRequest>), Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let active_cancel = Arc::new(Mutex::new(None));
        let generation = Arc::new(AtomicU64::new(0));
        let cache = Arc::new(Mutex::new(PreviewCache::default()));
        let worker_pending = Arc::clone(&pending);
        let worker_stop = Arc::clone(&stop);
        let worker_active_cancel = Arc::clone(&active_cancel);
        let worker_cache = Arc::clone(&cache);
        thread::Builder::new()
            .name("rovex-preview-worker".to_owned())
            .spawn(move || {
                loop {
                    let request = {
                        let (lock, condition) = &*worker_pending;
                        let Ok(pending) = lock.lock() else {
                            break;
                        };
                        let mut pending = match condition.wait_while(pending, |request| {
                            request.is_none() && !worker_stop.load(Ordering::Acquire)
                        }) {
                            Ok(pending) => pending,
                            Err(_) => break,
                        };
                        if pending.is_none() && worker_stop.load(Ordering::Acquire) {
                            break;
                        }
                        pending.take()
                    };
                    let Some(request) = request else {
                        continue;
                    };
                    if request.cancel.load(Ordering::Acquire) {
                        continue;
                    }
                    if let Ok(mut cache) = worker_cache.lock()
                        && let Some(preview) = cache.get(&request.path)
                    {
                        if !request.cancel.load(Ordering::Acquire) {
                            (request.callback)(PreviewEvent::Ready {
                                generation: request.generation,
                                preview,
                            });
                        }
                    } else {
                        let result = decode_thumbnail(&request.path, request.limits);
                        if request.cancel.load(Ordering::Acquire) {
                            continue;
                        }
                        match result {
                            Ok(preview) => {
                                if let Ok(mut cache) = worker_cache.lock() {
                                    cache.insert(&request.path, preview.clone());
                                }
                                (request.callback)(PreviewEvent::Ready {
                                    generation: request.generation,
                                    preview,
                                });
                            }
                            Err(error) => (request.callback)(PreviewEvent::Failed {
                                generation: request.generation,
                                error,
                            }),
                        }
                    }
                    if let Ok(mut active) = worker_active_cancel.lock()
                        && active
                            .as_ref()
                            .is_some_and(|current| Arc::ptr_eq(current, &request.cancel))
                    {
                        *active = None;
                    }
                }
            })
            .map_err(|_| ())?;
        Ok(Self {
            pending,
            stop,
            active_cancel,
            generation,
        })
    }

    pub(in crate::desktop) fn request<F>(
        &self,
        path: PathBuf,
        limits: PreviewLimits,
        callback: F,
    ) -> Result<u64, ()>
    where
        F: Fn(PreviewEvent) + Send + 'static,
    {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let cancel = Arc::new(AtomicBool::new(false));
        if let Ok(mut active) = self.active_cancel.lock()
            && let Some(previous) = active.replace(Arc::clone(&cancel))
        {
            previous.store(true, Ordering::Release);
        }
        let request = PreviewRequest {
            generation,
            path,
            limits,
            cancel,
            callback: Box::new(callback),
        };
        let (lock, condition) = &*self.pending;
        let Ok(mut pending) = lock.lock() else {
            return Err(());
        };
        *pending = Some(request);
        condition.notify_one();
        Ok(generation)
    }

    pub(in crate::desktop) fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Ok(active) = self.active_cancel.lock()
            && let Some(cancel) = active.as_ref()
        {
            cancel.store(true, Ordering::Release);
        }
        if let Ok(mut pending) = self.pending.0.lock()
            && let Some(request) = pending.take()
        {
            request.cancel.store(true, Ordering::Release);
        }
    }

    pub(in crate::desktop) fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
}

impl Drop for PreviewScheduler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.cancel();
        self.pending.1.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, ImageFormat, Rgba};
    use std::io::Cursor;
    use std::sync::mpsc;
    use std::time::Duration;

    fn write_png(path: &Path) {
        let image = ImageBuffer::from_pixel(4, 2, Rgba([90_u8, 30_u8, 180_u8, 255_u8]));
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("encode");
        fs::write(path, bytes.into_inner()).expect("write");
    }

    #[test]
    fn worker_emits_ready_and_reuses_cache() {
        let path =
            std::env::temp_dir().join(format!("rovex-preview-worker-{}.png", std::process::id()));
        write_png(&path);
        let scheduler = PreviewScheduler::new().expect("worker");
        let (sender, receiver) = mpsc::channel();
        scheduler
            .request(path.clone(), PreviewLimits::default(), move |event| {
                sender.send(event).expect("send");
            })
            .expect("request");
        let first = receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("first event");
        assert!(matches!(first, PreviewEvent::Ready { .. }));
        let (sender, receiver) = mpsc::channel();
        scheduler
            .request(path.clone(), PreviewLimits::default(), move |event| {
                sender.send(event).expect("send");
            })
            .expect("cached request");
        assert!(matches!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("cached event"),
            PreviewEvent::Ready { .. }
        ));
        fs::remove_file(path).expect("cleanup");
    }
}
