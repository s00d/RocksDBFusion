use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc};
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
    pub token: Option<String>
}

impl Request {
    fn parse_option<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.options
            .as_ref()
            .and_then(|opts| opts.get(key))
            .and_then(|value| value.parse::<T>().ok())
    }
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
    auth_token: Option<String>,  // Добавлено поле для токена
}

impl RocksDBServer {
    pub fn new(db_path: String, ttl_secs: Option<u64>, auth_token: Option<String>) -> Result<Self, String> {
        let db_manager = Arc::new(RocksDBManager::new(&*db_path.clone(), ttl_secs.clone())?);

        Ok(RocksDBServer {
            db_manager,
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
        if !self.is_authorized(&req) {
            return Response {
                success: false,
                result: None,
                error: Some("Unauthorized".to_string()),
            };
        }
        let key = req.key.clone();
        let value = req.value.clone();
        let cf_name = req.cf_name.clone();
        match req.action.as_str() {
            "put" => self.handle_put(key, value, cf_name).await,
            "get" => self.handle_get(key, cf_name, req.default).await,
            "delete" => self.handle_delete(key, cf_name).await,
            "merge" => self.handle_merge(key, value, cf_name).await,
            "keys" => self.handle_get_keys(&req).await,
            "list_column_families" => self.handle_list_column_families(value).await,
            "create_column_family" => self.handle_create_column_family(value).await,
            "drop_column_family" => self.handle_drop_column_family(value).await,
            "compact_range" => self.handle_compact_range(key, value, cf_name).await,
            "write_batch_put" => self.handle_write_batch_put(key, value, cf_name).await,
            "write_batch_merge" => self.handle_write_batch_merge(key, value, cf_name).await,
            "write_batch_delete" => self.handle_write_batch_delete(key, cf_name).await,
            "write_batch_write" => self.handle_write_batch_write().await,
            "write_batch_clear" => self.handle_write_batch_clear().await,
            "write_batch_destroy" => self.handle_write_batch_destroy().await,
            "create_iterator" => self.handle_create_iterator().await,
            "destroy_iterator" => self.handle_destroy_iterator(&req).await,
            "iterator_seek" => self.handle_iterator_seek(&req, key.unwrap_or_default(), rust_rocksdb::Direction::Forward).await,
            "iterator_seek_for_prev" => self.handle_iterator_seek(&req, key.unwrap_or_default(), rust_rocksdb::Direction::Reverse).await,
            "iterator_next" => self.handle_iterator_next(&req).await,
            "iterator_prev" => self.handle_iterator_prev(&req).await,
            "backup" => self.handle_backup().await,
            "restore_latest" => self.handle_restore_latest().await,
            "restore" => self.handle_restore_request(&req).await,
            "get_backup_info" => self.handle_get_backup_info().await,
            _ => Response { success: false, result: None, error: Some("Unknown action".to_string()) },
        }
    }

    fn is_authorized(&self, req: &Request) -> bool {
        match &self.auth_token {
            Some(auth_token) => req.token.as_deref() == Some(auth_token),
            None => true,
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

    async fn handle_get_keys(&self, req: &Request) -> Response {
        let start = req.parse_option::<usize>("start").unwrap_or(0);
        let limit = req.parse_option::<usize>("limit").unwrap_or(20);
        let query = req.options.as_ref().and_then(|opts| opts.get("query").cloned());

        self.db_manager.get_keys(start, limit, query)
            .map(|keys| {
                let result = serde_json::to_string(&keys).unwrap();
                Response { success: true, result: Some(result), error: None }
            })
            .unwrap_or_else(|e| Response { success: false, result: None, error: Some(e) })
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
        Response {
            success: true,
            result: Some(self.db_manager.create_iterator().to_string()),
            error: None,
        }
    }

    async fn handle_destroy_iterator(&self, req: &Request) -> Response {
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager.destroy_iterator(iterator_id)
            .map(|_| Response { success: true, result: None, error: None })
            .unwrap_or_else(|e| Response { success: false, result: None, error: Some(e) })
    }

    async fn handle_iterator_seek(&self, req: &Request, key: String, direction: rust_rocksdb::Direction) -> Response {
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager.iterator_seek(iterator_id, key, direction)
            .map(|result| Response { success: true, result: Some(result), error: None })
            .unwrap_or_else(|e| Response { success: false, result: None, error: Some(e) })
    }

    async fn handle_iterator_next(&self, req: &Request) -> Response {
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager.iterator_next(iterator_id)
            .map(|result| Response { success: true, result: Some(result), error: None })
            .unwrap_or_else(|e| Response { success: false, result: None, error: Some(e) })
    }

    async fn handle_iterator_prev(&self, req: &Request) -> Response {
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager.iterator_prev(iterator_id)
            .map(|result| Response { success: true, result: Some(result), error: None })
            .unwrap_or_else(|e| Response { success: false, result: None, error: Some(e) })
    }

    async fn handle_backup(&self) -> Response {
        match self.db_manager.backup() {
            Ok(_) => Response { success: true, result: Some("Backup created successfully".to_string()), error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }

    async fn handle_restore_latest(&self) -> Response {
        match self.db_manager.restore_latest_backup() {
            Ok(_) => Response { success: true, result: Some("Database restored from latest backup".to_string()), error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }

    async fn handle_restore_request(&self, req: &Request) -> Response {
        let backup_id = req.parse_option::<u32>("backup_id").unwrap_or(0);
        match self.db_manager.restore_backup(backup_id) {
            Ok(_) => Response { success: true, result: Some(format!("Database restored from backup {}", backup_id)), error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }


    async fn handle_get_backup_info(&self) -> Response {
        match self.db_manager.get_backup_info() {
            Ok(info) => {
                let result = serde_json::to_string(&info).unwrap();
                Response { success: true, result: Some(result), error: None }
            }
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    }
}