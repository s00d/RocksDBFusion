use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::db_manager::RocksDBManager;


#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub action: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub default: Option<String>,
    pub cf_name: Option<String>,
    pub options: Option<HashMap<String, String>>,
    pub iterator_id: Option<usize>,
    pub token: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub result: Option<String>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct RocksDBServer {
    db_manager: Arc<RocksDBManager>,
    iterators: Arc<Mutex<HashMap<usize, (Vec<u8>, rust_rocksdb::Direction)>>>,
    iterator_id_counter: Arc<AtomicUsize>,
    auth_token: Option<String>,  // Добавлено поле для токена
}

impl RocksDBServer {
    pub fn new(db_path: String, ttl_secs: Option<u64>, auth_token: Option<String>) -> Result<Self, String> {
        let db_manager = Arc::new(RocksDBManager::new(&*db_path.clone(), ttl_secs.clone())?);
        let iterators = Arc::new(Mutex::new(HashMap::new()));
        let iterator_id_counter = Arc::new(AtomicUsize::new(0));

        Ok(RocksDBServer {
            db_manager,
            iterators,
            iterator_id_counter,
            auth_token,
        })
    }

    pub async fn handle_client(
        &self,
        mut socket: tokio::net::TcpStream
    ) {
        let mut buffer = Vec::new();

        loop {
            match socket.read_buf(&mut buffer).await {
                Ok(0) => {
                    // Connection closed
                    break;
                }
                Ok(_) => {
                    if let Some(position) = buffer.iter().position(|&b| b == b'\n') {
                        let request_data = buffer[..position].to_vec(); // Копируем данные до позиции
                        buffer = buffer.split_off(position + 1); // Оставляем только оставшуюся часть буфера

                        match serde_json::from_slice::<Request>(&request_data) {
                            Ok(request) => {
                                let response = self.handle_request(request).await;
                                let mut response_bytes = serde_json::to_vec(&response).unwrap();
                                response_bytes.push(b'\n'); // Добавляем '\n' в конец ответа

                                if let Err(e) = socket.write_all(&response_bytes).await {
                                    eprintln!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize request: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from socket: {}", e);
                    break;
                }
            }
        }
    }

    async fn handle_request(&self, req: Request) -> Response {
        if let Some(ref auth_token) = self.auth_token {
            if req.token.as_deref() != Some(auth_token) {
                return Response {
                    success: false,
                    result: None,
                    error: Some("Unauthorized".to_string()),
                };
            }
        }
        match req.action.as_str() {
            "put" => self.handle_put(req.key, req.value, req.cf_name).await,
            "get" => self.handle_get(req.key, req.cf_name, req.default).await,
            "delete" => self.handle_delete(req.key, req.cf_name).await,
            "merge" => self.handle_merge(req.key, req.value, req.cf_name).await,
            "keys" => self.handle_get_keys(req.options).await,
            "list_column_families" => self.handle_list_column_families(req.value).await,
            "create_column_family" => self.handle_create_column_family(req.value).await,
            "drop_column_family" => self.handle_drop_column_family(req.value).await,
            "compact_range" => self.handle_compact_range(req.key, req.value, req.cf_name).await,
            "write_batch_put" => self.handle_write_batch_put(req.key, req.value, req.cf_name).await,
            "write_batch_merge" => self.handle_write_batch_merge(req.key, req.value, req.cf_name).await,
            "write_batch_delete" => self.handle_write_batch_delete(req.key, req.cf_name).await,
            "write_batch_write" => self.handle_write_batch_write().await,
            "write_batch_clear" => self.handle_write_batch_clear().await,
            "write_batch_destroy" => self.handle_write_batch_destroy().await,
            "create_iterator" => self.handle_create_iterator().await,
            "destroy_iterator" => self.handle_destroy_iterator(req.iterator_id.unwrap_or(0)).await,
            "iterator_seek" => self.handle_iterator_seek(req.iterator_id.unwrap_or(0), req.key.unwrap_or_default(), rust_rocksdb::Direction::Forward).await,
            "iterator_seek_for_prev" => self.handle_iterator_seek(req.iterator_id.unwrap_or(0), req.key.unwrap_or_default(), rust_rocksdb::Direction::Reverse).await,
            "iterator_next" => self.handle_iterator_next(req.iterator_id.unwrap_or(0)).await,
            "iterator_prev" => self.handle_iterator_prev(req.iterator_id.unwrap_or(0)).await,
            _ => Response { success: false, result: None, error: Some("Unknown action".to_string()) },
        }
    }

    async fn handle_put(
        &self,
        key: Option<String>,
        value: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match (key, value) {
            (Some(key), Some(value)) => match self.db_manager.put(key, value, cf_name) {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            },
            _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) },
        }
    }

    async fn handle_get(
        &self,
        key: Option<String>,
        cf_name: Option<String>,
        default: Option<String>
    ) -> Response {
        match key {
            Some(key) => match self.db_manager.get(key, cf_name, default) {
                Ok(Some(value)) => Response { success: true, result: Some(value), error: None },
                Ok(None) => Response { success: false, result: None, error: Some("Key not found".to_string()) },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            },
            None => Response { success: false, result: None, error: Some("Missing key".to_string()) },
        }
    }

    async fn handle_delete(
        &self,
        key: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match key {
            Some(key) => match self.db_manager.delete(key, cf_name) {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            },
            None => Response { success: false, result: None, error: Some("Missing key".to_string()) },
        }
    }

    async fn handle_merge(
        &self,
        key: Option<String>,
        value: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match (key, value) {
            (Some(key), Some(value)) => match self.db_manager.merge(key, value, cf_name) {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            },
            _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) },
        }
    }

    async fn handle_get_keys(&self, options: Option<HashMap<String, String>>) -> Response {
        let start = options
            .as_ref()
            .and_then(|opts| opts.get("start").and_then(|s| s.parse::<usize>().ok()))
            .unwrap_or(0);
        let limit = options
            .as_ref()
            .and_then(|opts| opts.get("limit").and_then(|l| l.parse::<usize>().ok()))
            .unwrap_or(20);
        let query = options.as_ref().and_then(|opts| opts.get("query").cloned());

        match self.db_manager.get_keys(start, limit, query) {
            Ok(keys) => {
                let result = serde_json::to_string(&keys).unwrap();
                Response { success: true, result: Some(result), error: None }
            }
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }

    async fn handle_list_column_families(&self, path: Option<String>) -> Response {
        match path {
            Some(path) => {
                match self.db_manager.list_column_families(path) {
                    Ok(cfs) => Response { success: true, result: Some(serde_json::to_string(&cfs).unwrap()), error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
                }
            }
            None => Response { success: false, result: None, error: Some("Missing path".to_string()) }
        }
    }

    async fn handle_create_column_family(
        &self,
        cf_name: Option<String>
    ) -> Response {
        match cf_name {
            Some(cf_name) => match self.db_manager.create_column_family(cf_name) {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            },
            None => Response { success: false, result: None, error: Some("Missing column family name".to_string()) },
        }
    }

    async fn handle_drop_column_family(
        &self,
        cf_name: Option<String>
    ) -> Response {
        match cf_name {
            Some(cf_name) => match self.db_manager.drop_column_family(cf_name) {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            },
            None => Response { success: false, result: None, error: Some("Missing column family name".to_string()) },
        }
    }

    async fn handle_compact_range(
        &self,
        start: Option<String>,
        end: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match self.db_manager.compact_range(start, end, cf_name) {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }

    async fn handle_write_batch_put(&self, key: Option<String>, value: Option<String>, cf_name: Option<String>) -> Response {
        match (key, value) {
            (Some(key), Some(value)) => {
                match self.db_manager.write_batch_put(key, value, cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e) },
                }
            }
            _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
        }
    }

    async fn handle_write_batch_merge(&self, key: Option<String>, value: Option<String>, cf_name: Option<String>) -> Response {
        match (key, value) {
            (Some(key), Some(value)) => {
                match self.db_manager.write_batch_merge(key, value, cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e) },
                }
            }
            _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
        }
    }

    async fn handle_write_batch_delete(&self, key: Option<String>, cf_name: Option<String>) -> Response {
        match key {
            Some(key) => {
                match self.db_manager.write_batch_delete(key, cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e) },
                }
            }
            None => Response { success: false, result: None, error: Some("Missing key".to_string()) }
        }
    }

    async fn handle_write_batch_write(&self) -> Response {
        match self.db_manager.write_batch_write() {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }

    async fn handle_write_batch_clear(&self) -> Response {
        match self.db_manager.write_batch_clear() {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }

    async fn handle_write_batch_destroy(&self) -> Response {
        match self.db_manager.write_batch_destroy() {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(_) => Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) },
        }
    }

    async fn handle_create_iterator(&self) -> Response {
        let mut iterators = self.iterators.lock().unwrap();
        let id = self.iterator_id_counter.fetch_add(1, Ordering::SeqCst);
        iterators.insert(id, (vec![], rust_rocksdb::Direction::Forward));
        Response { success: true, result: Some(id.to_string()), error: None }
    }

    async fn handle_destroy_iterator(&self, iterator_id: usize) -> Response {
        let mut iterators = self.iterators.lock().unwrap();
        if iterators.remove(&iterator_id).is_some() {
            Response { success: true, result: None, error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator ID not found".to_string()) }
        }
    }

    async fn handle_iterator_seek(&self, iterator_id: usize, key: String, direction: rust_rocksdb::Direction) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        if let Some(ref db) = *db {
            let mut iterators = self.iterators.lock().unwrap();
            if let Some(iterator) = iterators.get_mut(&iterator_id) {
                let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(key.as_bytes(), direction));
                if let Some(Ok((k, _))) = iter.next() {
                    iterator.0 = k.to_vec();
                    iterator.1 = direction;
                    Response { success: true, result: Some(String::from_utf8(k.to_vec()).unwrap()), error: None }
                } else {
                    Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
                }
            } else {
                Response { success: false, result: None, error: Some("Iterator ID not found".to_string()) }
            }
        } else {
            Response { success: false, result: None, error: Some("Database is not open".to_string()) }
        }
    }

    async fn handle_iterator_next(&self, iterator_id: usize) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iterators = self.iterators.lock().unwrap();
        if let Some(ref db) = *db {
            if let Some(iterator) = iterators.get_mut(&iterator_id) {
                let (ref mut pos, direction) = *iterator;
                let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(pos, direction));
                iter.next(); // Move to current position
                if let Some(Ok((k, v))) = iter.next() {
                    pos.clone_from_slice(&*k);
                    Response { success: true, result: Some(format!("{}:{}", String::from_utf8(k.to_vec()).unwrap(), String::from_utf8(v.to_vec()).unwrap())), error: None }
                } else {
                    Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
                }
            } else {
                Response { success: false, result: None, error: Some("Iterator ID not found".to_string()) }
            }
        } else {
            Response { success: false, result: None, error: Some("Database is not open".to_string()) }
        }

    }

    async fn handle_iterator_prev(&self, iterator_id: usize) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        if let Some(ref db) = *db {
            let mut iterators = self.iterators.lock().unwrap();
            if let Some(iterator) = iterators.get_mut(&iterator_id) {
                let (ref mut pos, _direction) = *iterator;
                let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(pos, rust_rocksdb::Direction::Reverse));
                iter.next(); // Move to current position
                if let Some(Ok((k, v))) = iter.next() {
                    pos.clone_from_slice(&*k);
                    Response { success: true, result: Some(format!("{}:{}", String::from_utf8(k.to_vec()).unwrap(), String::from_utf8(v.to_vec()).unwrap())), error: None }
                } else {
                    Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
                }
            } else {
                Response { success: false, result: None, error: Some("Iterator ID not found".to_string()) }
            }
        } else {
            Response { success: false, result: None, error: Some("Database is not open".to_string()) }
        }
    }
}