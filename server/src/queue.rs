use async_std::channel::{unbounded, Receiver, Sender};
use async_std::sync::{Arc};
use log::error;
use crate::db_manager::RocksDBManager;

pub enum TaskType {
    Put,
    Delete,
}

struct Task {
    task_type: TaskType,
    key: String,
    value: Option<String>,
    cf_name: Option<String>,
}

pub(crate) struct TaskQueue {
    sender: Sender<Task>,
    receiver: Receiver<Task>,
}

impl TaskQueue {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = unbounded();
        TaskQueue { sender, receiver }
    }

    pub(crate) async fn add_task(&self, task_type: TaskType, key: String, value: Option<String>, cf_name: Option<String>) {
        self.sender.send(Task { key, value, cf_name, task_type }).await.unwrap();
    }

    pub(crate) async fn process_tasks(&self, db_manager: Arc<RocksDBManager>) {
        while let Ok(task) = self.receiver.recv().await {
            match task.task_type {
                TaskType::Put => {
                    if let Some(value) = task.value {
                        if let Err(e) = db_manager.put(task.key.clone(), value.clone(), task.cf_name.clone(), None) {
                            error!("Failed to persist data to RocksDB: {}", e);
                        }
                    }
                },
                TaskType::Delete => {
                    if let Err(e) = db_manager.delete(task.key.clone(), task.cf_name.clone(), None) {
                        error!("Failed to delete data from RocksDB: {}", e);
                    }
                },
            }
        }
    }
}