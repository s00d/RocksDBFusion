use crate::db_manager::RocksDBManager;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub action: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub default: Option<String>,
    pub cf_name: Option<String>,
    pub options: Option<HashMap<String, String>>,
    pub token: Option<String>,
    pub txn_id: Option<usize>,
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
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct RocksDBServer {
    db_manager: Arc<RocksDBManager>,
    auth_token: Option<String>,
}

impl RocksDBServer {
    pub fn new(
        db_path: String,
        ttl_secs: Option<u64>,
        auth_token: Option<String>,
    ) -> Result<Self, String> {
        let db_manager = Arc::new(RocksDBManager::new(&db_path, ttl_secs)?);

        Ok(RocksDBServer {
            db_manager,
            auth_token,
        })
    }

    pub async fn handle_client(&self, mut socket: tokio::net::TcpStream) {
        let mut buffer = Vec::new();

        loop {
            match socket.read_buf(&mut buffer).await {
                Ok(0) => {
                    info!("Connection closed");
                    break;
                }
                Ok(_) => {
                    if let Some(position) = buffer.iter().position(|&b| b == b'\n') {
                        let request_data = buffer[..position].to_vec(); // Copy data up to position
                        buffer = buffer.split_off(position + 1); // Leave only the remaining part of the buffer

                        match serde_json::from_slice::<Request>(&request_data) {
                            Ok(request) => {
                                debug!("Received request: {:?}", request);
                                let response = self.handle_request(request).await;
                                let mut response_bytes = serde_json::to_vec(&response).unwrap();
                                response_bytes.push(b'\n'); // Add '\n' to the end of the response

                                if let Err(e) = socket.write_all(&response_bytes).await {
                                    error!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize request: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from socket: {}", e);
                    break;
                }
            }
        }
    }

    async fn handle_request(&self, req: Request) -> Response {
        if !self.is_authorized(&req) {
            error!("Unauthorized request: {:?}", req);
            return Response {
                success: false,
                result: None,
                error: Some("Unauthorized".to_string()),
            };
        }
        debug!("Handling request action: {}", req.action);
        match req.action.as_str() {
            "put" => self.handle_put(req).await,
            "get" => self.handle_get(req).await,
            "delete" => self.handle_delete(req).await,
            "merge" => self.handle_merge(req).await,
            "get_property" => self.handle_get_property(req).await,
            "keys" => self.handle_get_keys(req).await,
            "all" => self.handle_get_all(req).await,
            "list_column_families" => self.handle_list_column_families(req).await,
            "create_column_family" => self.handle_create_column_family(req).await,
            "drop_column_family" => self.handle_drop_column_family(req).await,
            "compact_range" => self.handle_compact_range(req).await,
            "write_batch_put" => self.handle_write_batch_put(req).await,
            "write_batch_merge" => self.handle_write_batch_merge(req).await,
            "write_batch_delete" => self.handle_write_batch_delete(req).await,
            "write_batch_write" => self.handle_write_batch_write().await,
            "write_batch_clear" => self.handle_write_batch_clear().await,
            "write_batch_destroy" => self.handle_write_batch_destroy().await,
            "create_iterator" => self.handle_create_iterator().await,
            "destroy_iterator" => self.handle_destroy_iterator(req).await,
            "iterator_seek" => {
                self.handle_iterator_seek(req, rust_rocksdb::Direction::Forward)
                    .await
            }
            "iterator_seek_for_prev" => {
                self.handle_iterator_seek(req, rust_rocksdb::Direction::Reverse)
                    .await
            }
            "iterator_next" => self.handle_iterator_next(req).await,
            "iterator_prev" => self.handle_iterator_prev(req).await,
            "backup" => self.handle_backup().await,
            "restore_latest" => self.handle_restore_latest().await,
            "restore" => self.handle_restore_request(req).await,
            "get_backup_info" => self.handle_get_backup_info().await,
            "begin_transaction" => self.handle_begin_transaction().await,
            "commit_transaction" => self.handle_commit_transaction(req).await,
            "rollback_transaction" => self.handle_rollback_transaction(req).await,
            _ => Response {
                success: false,
                result: None,
                error: Some("Unknown action".to_string()),
            },
        }
    }

    fn is_authorized(&self, req: &Request) -> bool {
        match &self.auth_token {
            Some(auth_token) => req.token.as_deref() == Some(auth_token),
            None => true,
        }
    }

    async fn handle_put(&self, req: Request) -> Response {
        debug!("handle_put with key: {:?}, value: {:?}", req.key, req.value);
        match (req.key, req.value) {
            (Some(key), Some(value)) => match self.db_manager.put(key, value, req.cf_name, req.txn_id) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            _ => Response {
                success: false,
                result: None,
                error: Some("Missing key or value".to_string()),
            },
        }
    }

    async fn handle_get(&self, req: Request) -> Response {
        debug!("handle_get with key: {:?}", req.key);
        match req.key {
            Some(key) => match self.db_manager.get(key, req.cf_name, req.default, req.txn_id) {
                Ok(Some(value)) => Response {
                    success: true,
                    result: Some(value),
                    error: None,
                },
                Ok(None) => Response {
                    success: false,
                    result: None,
                    error: Some("Key not found".to_string()),
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing key".to_string()),
            },
        }
    }

    async fn handle_delete(&self, req: Request) -> Response {
        debug!("handle_delete with key: {:?}", req.key);
        match req.key {
            Some(key) => match self.db_manager.delete(key, req.cf_name, req.txn_id) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing key".to_string()),
            },
        }
    }

    async fn handle_merge(&self, req: Request) -> Response {
        debug!(
            "handle_merge with key: {:?}, value: {:?}",
            req.key, req.value
        );
        match (req.key, req.value) {
            (Some(key), Some(value)) => match self.db_manager.merge(key, value, req.cf_name, req.txn_id) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            _ => Response {
                success: false,
                result: None,
                error: Some("Missing key or value".to_string()),
            },
        }
    }

    async fn handle_get_property(&self, req: Request) -> Response {
        debug!("handle_get_property with property: {:?}", req.value);
        match req.value {
            Some(value) => match self.db_manager.get_property(value, req.cf_name) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            _ => Response {
                success: false,
                result: None,
                error: Some("Missing key or value".to_string()),
            },
        }
    }

    async fn handle_get_keys(&self, req: Request) -> Response {
        debug!("handle_get_keys with options: {:?}", req.options);
        let start = req.parse_option::<usize>("start").unwrap_or(0);
        let limit = req.parse_option::<usize>("limit").unwrap_or(20);
        let query = req
            .options
            .as_ref()
            .and_then(|opts| opts.get("query").cloned());

        self.db_manager
            .get_keys(start, limit, query)
            .map(|keys| {
                let result = serde_json::to_string(&keys).unwrap();
                Response {
                    success: true,
                    result: Some(result),
                    error: None,
                }
            })
            .unwrap_or_else(|e| Response {
                success: false,
                result: None,
                error: Some(e),
            })
    }

    async fn handle_get_all(&self, req: Request) -> Response {
        debug!("handle_get_keys with options: {:?}", req.options);
        let query = req
            .options
            .as_ref()
            .and_then(|opts| opts.get("query").cloned());

        self.db_manager
            .get_all(query)
            .map(|keys| {
                let result = serde_json::to_string(&keys).unwrap();
                Response {
                    success: true,
                    result: Some(result),
                    error: None,
                }
            })
            .unwrap_or_else(|e| Response {
                success: false,
                result: None,
                error: Some(e),
            })
    }

    async fn handle_list_column_families(&self, req: Request) -> Response {
        debug!("handle_list_column_families with value: {:?}", req.value);
        match req.value {
            Some(path) => match self.db_manager.list_column_families(path) {
                Ok(cfs) => Response {
                    success: true,
                    result: Some(serde_json::to_string(&cfs).unwrap()),
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing path".to_string()),
            },
        }
    }

    async fn handle_create_column_family(&self, req: Request) -> Response {
        debug!(
            "handle_create_column_family with cf_name: {:?}",
            req.cf_name
        );
        match req.cf_name {
            Some(cf_name) => match self.db_manager.create_column_family(cf_name) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing column family name".to_string()),
            },
        }
    }

    async fn handle_drop_column_family(&self, req: Request) -> Response {
        debug!("handle_drop_column_family with cf_name: {:?}", req.cf_name);
        match req.cf_name {
            Some(cf_name) => match self.db_manager.drop_column_family(cf_name) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing column family name".to_string()),
            },
        }
    }

    async fn handle_compact_range(&self, req: Request) -> Response {
        debug!("handle_compact_range with options: {:?}", req.options);
        let start = req
            .parse_option::<String>("start")
            .unwrap_or("".to_string());
        let end = req.parse_option::<String>("end").unwrap_or("".to_string());
        match self
            .db_manager
            .compact_range(Some(start), Some(end), req.cf_name)
        {
            Ok(_) => Response {
                success: true,
                result: None,
                error: None,
            },
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_write_batch_put(&self, req: Request) -> Response {
        debug!(
            "handle_write_batch_put with key: {:?}, value: {:?}",
            req.key, req.value
        );
        match (req.key, req.value) {
            (Some(key), Some(value)) => {
                match self.db_manager.write_batch_put(key, value, req.cf_name) {
                    Ok(_) => Response {
                        success: true,
                        result: None,
                        error: None,
                    },
                    Err(e) => Response {
                        success: false,
                        result: None,
                        error: Some(e),
                    },
                }
            }
            _ => Response {
                success: false,
                result: None,
                error: Some("Missing key or value".to_string()),
            },
        }
    }

    async fn handle_write_batch_merge(&self, req: Request) -> Response {
        debug!(
            "handle_write_batch_merge with key: {:?}, value: {:?}",
            req.key, req.value
        );
        match (req.key, req.value) {
            (Some(key), Some(value)) => {
                match self.db_manager.write_batch_merge(key, value, req.cf_name) {
                    Ok(_) => Response {
                        success: true,
                        result: None,
                        error: None,
                    },
                    Err(e) => Response {
                        success: false,
                        result: None,
                        error: Some(e),
                    },
                }
            }
            _ => Response {
                success: false,
                result: None,
                error: Some("Missing key or value".to_string()),
            },
        }
    }

    async fn handle_write_batch_delete(&self, req: Request) -> Response {
        debug!("handle_write_batch_delete with key: {:?}", req.key);
        match req.key {
            Some(key) => match self.db_manager.write_batch_delete(key, req.cf_name) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing key".to_string()),
            },
        }
    }

    async fn handle_write_batch_write(&self) -> Response {
        debug!("handle_write_batch_write");
        match self.db_manager.write_batch_write() {
            Ok(_) => Response {
                success: true,
                result: None,
                error: None,
            },
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_write_batch_clear(&self) -> Response {
        debug!("handle_write_batch_clear");
        match self.db_manager.write_batch_clear() {
            Ok(_) => Response {
                success: true,
                result: None,
                error: None,
            },
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_write_batch_destroy(&self) -> Response {
        debug!("handle_write_batch_destroy");
        match self.db_manager.write_batch_destroy() {
            Ok(_) => Response {
                success: true,
                result: None,
                error: None,
            },
            Err(_) => Response {
                success: false,
                result: None,
                error: Some("WriteBatch not initialized".to_string()),
            },
        }
    }

    async fn handle_create_iterator(&self) -> Response {
        debug!("handle_create_iterator");
        Response {
            success: true,
            result: Some(self.db_manager.create_iterator().to_string()),
            error: None,
        }
    }

    async fn handle_destroy_iterator(&self, req: Request) -> Response {
        debug!(
            "handle_destroy_iterator with iterator_id: {:?}",
            req.parse_option::<usize>("iterator_id")
        );
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager
            .destroy_iterator(iterator_id)
            .map(|_| Response {
                success: true,
                result: None,
                error: None,
            })
            .unwrap_or_else(|e| Response {
                success: false,
                result: None,
                error: Some(e),
            })
    }

    async fn handle_iterator_seek(
        &self,
        req: Request,
        direction: rust_rocksdb::Direction,
    ) -> Response {
        debug!(
            "handle_iterator_seek with iterator_id: {:?}, key: {:?}",
            req.parse_option::<usize>("iterator_id"),
            req.key
        );
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        match req.key {
            Some(key) => self
                .db_manager
                .iterator_seek(iterator_id, key, direction)
                .map(|result| Response {
                    success: true,
                    result: Some(result),
                    error: None,
                })
                .unwrap_or_else(|e| Response {
                    success: false,
                    result: None,
                    error: Some(e),
                }),
            None => Response {
                success: false,
                result: None,
                error: Some("Missing key".to_string()),
            },
        }
    }

    async fn handle_iterator_next(&self, req: Request) -> Response {
        debug!(
            "handle_iterator_next with iterator_id: {:?}",
            req.parse_option::<usize>("iterator_id")
        );
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager
            .iterator_next(iterator_id)
            .map(|result| Response {
                success: true,
                result: Some(result),
                error: None,
            })
            .unwrap_or_else(|e| Response {
                success: false,
                result: None,
                error: Some(e),
            })
    }

    async fn handle_iterator_prev(&self, req: Request) -> Response {
        debug!(
            "handle_iterator_prev with iterator_id: {:?}",
            req.parse_option::<usize>("iterator_id")
        );
        let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
        self.db_manager
            .iterator_prev(iterator_id)
            .map(|result| Response {
                success: true,
                result: Some(result),
                error: None,
            })
            .unwrap_or_else(|e| Response {
                success: false,
                result: None,
                error: Some(e),
            })
    }

    async fn handle_backup(&self) -> Response {
        debug!("handle_backup");
        match self.db_manager.backup() {
            Ok(_) => Response {
                success: true,
                result: Some("Backup created successfully".to_string()),
                error: None,
            },
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_restore_latest(&self) -> Response {
        debug!("handle_restore_latest");
        match self.db_manager.restore_latest_backup() {
            Ok(_) => Response {
                success: true,
                result: Some("Database restored from latest backup".to_string()),
                error: None,
            },
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_restore_request(&self, req: Request) -> Response {
        debug!(
            "handle_restore_request with backup_id: {:?}",
            req.parse_option::<u32>("backup_id")
        );
        let backup_id = req.parse_option::<u32>("backup_id").unwrap_or(0);
        match self.db_manager.restore_backup(backup_id) {
            Ok(_) => Response {
                success: true,
                result: Some(format!("Database restored from backup {}", backup_id)),
                error: None,
            },
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_get_backup_info(&self) -> Response {
        debug!("handle_get_backup_info");
        match self.db_manager.get_backup_info() {
            Ok(info) => {
                let result = serde_json::to_string(&info).unwrap();
                Response {
                    success: true,
                    result: Some(result),
                    error: None,
                }
            }
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_begin_transaction(&self) -> Response {
        debug!("handle_begin_transaction");
        match self.db_manager.begin_transaction() {
            Ok(info) => {
                let result = serde_json::to_string(&info).unwrap();
                Response {
                    success: true,
                    result: Some(result),
                    error: None,
                }
            }
            Err(e) => Response {
                success: false,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_commit_transaction(&self, req: Request) -> Response {
        debug!("handle_commit_transaction, txn_id: {:?}", req.txn_id);

        match req.txn_id {
            Some(txn_id) => match self.db_manager.commit_transaction(txn_id) {
                Ok(info) => {
                    let result = serde_json::to_string(&info).unwrap();
                    Response {
                        success: true,
                        result: Some(result),
                        error: None,
                    }
                }
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing txn_id".to_string()),
            },
        }
    }

    async fn handle_rollback_transaction(&self, req: Request) -> Response {
        debug!("handle_rollback_transaction, txn_id: {:?}", req.txn_id);

        match req.txn_id {
            Some(txn_id) => match self.db_manager.rollback_transaction(txn_id) {
                Ok(info) => {
                    let result = serde_json::to_string(&info).unwrap();
                    Response {
                        success: true,
                        result: Some(result),
                        error: None,
                    }
                }
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            },
            None => Response {
                success: false,
                result: None,
                error: Some("Missing txn_id".to_string()),
            },
        }
    }
}
