use crate::cache::queue::{TaskQueue, TaskType};
use crate::db_manager::RocksDBManager;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::metrics::METRICS;

type CacheData = Arc<RwLock<HashMap<(String, Option<String>), (String, Instant)>>>;

pub(crate) struct CacheLayer {
    data: CacheData,
    ttl: Duration,
    pub(crate) enabled: bool,
    task_queue: Arc<TaskQueue>,
}

impl CacheLayer {
    pub(crate) fn new(ttl: Duration, enabled: bool, db_manager: Arc<RocksDBManager>) -> Self {
        let data = Arc::new(RwLock::new(HashMap::new()));
        let task_queue = Arc::new(TaskQueue::new());

        let queue_clone = task_queue.clone();
        if enabled {
            task::spawn(async move {
                queue_clone.process_tasks(db_manager).await;
            });
        }

        let cache = CacheLayer {
            data: data.clone(),
            ttl,
            enabled,
            task_queue,
        };

        if enabled {
            let cache_clone = cache.clone();
            task::spawn(async move {
                loop {
                    task::sleep(Duration::from_secs(60)).await;
                    cache_clone.cleanup().await;
                }
            });
        }

        cache
    }

    pub(crate) async fn get(&self, key: &str, cf_name: Option<String>) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let mut data = self.data.write().await;
        if let Some((value, expires_at)) = data.get_mut(&(key.to_string(), cf_name)) {
            *expires_at = Instant::now() + self.ttl;
            METRICS.inc_cache_hits();
            return Some(value.clone());
        }
        METRICS.inc_cache_misses();
        None
    }

    pub(crate) async fn put(&self, key: String, value: String, cf_name: Option<String>) {
        if self.enabled {
            let mut data = self.data.write().await;
            let expires_at = Instant::now() + self.ttl;
            data.insert((key.clone(), cf_name.clone()), (value.clone(), expires_at));
            METRICS.inc_cache_set();
            self.task_queue
                .add_task(TaskType::Put, key, Some(value), cf_name)
                .await;
        }
    }

    pub(crate) async fn delete(&self, key: String, cf_name: Option<String>) {
        if self.enabled {
            let mut data = self.data.write().await;
            data.remove(&(key.clone(), cf_name.clone()));
            self.task_queue
                .add_task(TaskType::Delete, key, None, cf_name)
                .await;
        }
    }

    pub(crate) async fn clear(&self, key: String, cf_name: Option<String>) {
        if self.enabled {
            let mut data = self.data.write().await;
            data.remove(&(key.clone(), cf_name.clone()));
        }
    }

    async fn cleanup(&self) {
        let mut data = self.data.write().await;
        let now = Instant::now();
        data.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

impl Clone for CacheLayer {
    fn clone(&self) -> Self {
        CacheLayer {
            data: self.data.clone(),
            ttl: self.ttl,
            enabled: self.enabled,
            task_queue: self.task_queue.clone(),
        }
    }
}
