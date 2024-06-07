mod transaction;
mod db_manager;
mod backup_manager;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use std::sync::{Arc, Mutex};
use bytes::BytesMut;
use tokio_stream::StreamExt;
use futures::SinkExt;
use crate::transaction::RocksDBTransaction;
use crate::db_manager::RocksDBManager;
use crate::backup_manager::RocksDBBackupManager;


#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    action: String,
    key: Option<String>,
    value: Option<String>,
    cf_name: Option<String>,
    options: Option<HashMap<String, String>>,
    backup_path: Option<String>,
    num_backups_to_keep: Option<usize>,
    backup_id: Option<u32>,
    restore_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct RocksDBServer {
    db_manager: Arc<RocksDBManager>,
    backup_manager: RocksDBBackupManager,
    txn_db: Arc<Mutex<Option<RocksDBTransaction>>>,
}

impl RocksDBServer {
    pub fn new(db_path: String, backup_path: String, ttl_secs: Option<u64>) -> Result<Self, String> {
        let db_manager = Arc::new(RocksDBManager::new(&*db_path.clone(), ttl_secs.clone())?);
        let backup_manager = RocksDBBackupManager::new(&backup_path)?;
        let txn_db: Arc<Mutex<Option<RocksDBTransaction>>> = Arc::new(Mutex::new(None));

        Ok(RocksDBServer {
            db_manager,
            backup_manager,
            txn_db,
        })
    }

    pub async fn handle_client(
        &self,
        mut socket: Framed<TcpStream, LengthDelimitedCodec>
    ) {
        while let Some(Ok(frame)) = socket.next().await {
            let req: Request = serde_json::from_slice(&frame).unwrap();
            let response = match req.action.as_str() {
                "put" => self.handle_put(req.key, req.value, req.cf_name).await,
                "get" => self.handle_get(req.key, req.cf_name).await,
                "delete" => self.handle_delete(req.key, req.cf_name).await,
                "merge" => self.handle_merge(req.key, req.value, req.cf_name).await,
                "list_column_families" => self.handle_list_column_families(req.value).await,
                "create_column_family" => self.handle_create_column_family(req.value).await,
                "drop_column_family" => self.handle_drop_column_family(req.value).await,
                "compact_range" => self.handle_compact_range(req.key, req.value, req.cf_name).await,
                "begin_transaction" => self.handle_begin_transaction().await,
                "commit_transaction" => self.handle_commit_transaction().await,
                "rollback_transaction" => self.handle_rollback_transaction().await,
                "transaction_put" => self.handle_transaction_put(req.key, req.value, req.cf_name).await,
                "transaction_get" => self.handle_transaction_get(req.key, req.cf_name).await,
                "transaction_delete" => self.handle_transaction_delete(req.key, req.cf_name).await,
                "transaction_merge" => self.handle_transaction_merge(req.key, req.value, req.cf_name).await,
                "set_savepoint" => self.handle_set_savepoint().await,
                "rollback_to_savepoint" => self.handle_rollback_to_savepoint().await,
                "backup_create" => self.handle_backup_create().await,
                "backup_info" => self.handle_backup_info().await,
                "backup_purge_old" => self.handle_backup_purge_old(req.num_backups_to_keep).await,
                "backup_restore" => self.handle_backup_restore(req.backup_id, req.restore_path).await,
                "write_batch_put" => self.handle_write_batch_put(req.key, req.value, req.cf_name).await,
                "write_batch_merge" => self.handle_write_batch_merge(req.key, req.value, req.cf_name).await,
                "write_batch_delete" => self.handle_write_batch_delete(req.key, req.cf_name).await,
                "write_batch_write" => self.handle_write_batch_write().await,
                "write_batch_clear" => self.handle_write_batch_clear().await,
                "write_batch_destroy" => self.handle_write_batch_destroy().await,
                "seek_to_first" => self.handle_seek_to_first().await,
                "seek_to_last" => self.handle_seek_to_last().await,
                "seek" => self.handle_seek(req.key.unwrap_or_default()).await,
                "seek_for_prev" => self.handle_seek_for_prev(req.key.unwrap_or_default()).await,
                "valid" => self.handle_valid().await,
                "next" => self.handle_next().await,
                "prev" => self.handle_prev().await,
                _ => Response { success: false, result: None, error: Some("Unknown action".to_string()) },
            };
            let response_bytes = serde_json::to_vec(&response).unwrap();
            socket.send(BytesMut::from(&response_bytes[..]).into()).await.unwrap();
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
        cf_name: Option<String>
    ) -> Response {
        match key {
            Some(key) => match self.db_manager.get(key, cf_name) {
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

    async fn handle_begin_transaction(&self) -> Response {
        let mut txn_db = self.txn_db.lock().unwrap();
        if txn_db.is_none() {
            *txn_db = Some(RocksDBTransaction::new("path_to_your_db".to_string()).unwrap());
            Response { success: true, result: None, error: None }
        } else {
            Response { success: false, result: None, error: Some("Transaction already in progress".to_string()) }
        }
    }

    async fn handle_commit_transaction(&self) -> Response {
        let mut txn_db = self.txn_db.lock().unwrap();
        if let Some(ref txn) = *txn_db {
            match txn.commit() {
                Ok(_) => {
                    *txn_db = None;
                    Response { success: true, result: None, error: None }
                },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            }
        } else {
            Response { success: false, result: None, error: Some("No active transaction".to_string()) }
        }
    }

    async fn handle_rollback_transaction(&self) -> Response {
        let mut txn_db = self.txn_db.lock().unwrap();
        if let Some(ref txn) = *txn_db {
            match txn.rollback() {
                Ok(_) => {
                    *txn_db = None;
                    Response { success: true, result: None, error: None }
                },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            }
        } else {
            Response { success: false, result: None, error: Some("No active transaction".to_string()) }
        }
    }

    async fn handle_transaction_put(
        &self,
        key: Option<String>,
        value: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match (key, value) {
            (Some(key), Some(value)) => {
                let txn_db = self.txn_db.lock().unwrap();
                if let Some(ref txn) = *txn_db {
                    match txn.put(key, value, cf_name) {
                        Ok(_) => Response { success: true, result: None, error: None },
                        Err(e) => Response { success: false, result: None, error: Some(e) },
                    }
                } else {
                    Response { success: false, result: None, error: Some("No active transaction".to_string()) }
                }
            }
            _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
        }
    }

    async fn handle_transaction_get(
        &self,
        key: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match key {
            Some(key) => {
                let txn_db = self.txn_db.lock().unwrap();
                if let Some(ref txn) = *txn_db {
                    match txn.get(key, cf_name) {
                        Ok(Some(value)) => Response { success: true, result: Some(value), error: None },
                        Ok(None) => Response { success: false, result: None, error: Some("Key not found".to_string()) },
                        Err(e) => Response { success: false, result: None, error: Some(e) },
                    }
                } else {
                    Response { success: false, result: None, error: Some("No active transaction".to_string()) }
                }
            }
            None => Response { success: false, result: None, error: Some("Missing key".to_string()) }
        }
    }

    async fn handle_transaction_delete(
        &self,
        key: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match key {
            Some(key) => {
                let txn_db = self.txn_db.lock().unwrap();
                if let Some(ref txn) = *txn_db {
                    match txn.delete(key, cf_name) {
                        Ok(_) => Response { success: true, result: None, error: None },
                        Err(e) => Response { success: false, result: None, error: Some(e) },
                    }
                } else {
                    Response { success: false, result: None, error: Some("No active transaction".to_string()) }
                }
            }
            None => Response { success: false, result: None, error: Some("Missing key".to_string()) }
        }
    }

    async fn handle_transaction_merge(
        &self,
        key: Option<String>,
        value: Option<String>,
        cf_name: Option<String>
    ) -> Response {
        match (key, value) {
            (Some(key), Some(value)) => {
                let txn_db = self.txn_db.lock().unwrap();
                if let Some(ref txn) = *txn_db {
                    match txn.merge(key, value, cf_name) {
                        Ok(_) => Response { success: true, result: None, error: None },
                        Err(e) => Response { success: false, result: None, error: Some(e) },
                    }
                } else {
                    Response { success: false, result: None, error: Some("No active transaction".to_string()) }
                }
            }
            _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
        }
    }

    async fn handle_set_savepoint(&self) -> Response {
        let txn_db = self.txn_db.lock().unwrap();
        if let Some(ref txn) = *txn_db {
            txn.set_savepoint();
            Response { success: true, result: None, error: None }
        } else {
            Response { success: false, result: None, error: Some("No active transaction".to_string()) }
        }
    }

    async fn handle_rollback_to_savepoint(&self) -> Response {
        let txn_db = self.txn_db.lock().unwrap();
        if let Some(ref txn) = *txn_db {
            match txn.rollback_to_savepoint() {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e) },
            }
        } else {
            Response { success: false, result: None, error: Some("No active transaction".to_string()) }
        }
    }

    async fn handle_backup_create(&self) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        match self.backup_manager.create_backup(&*db) {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
        }
    }

    async fn handle_backup_info(&self) -> Response {
        match self.backup_manager.get_backup_info() {
            Ok(info) => {
                let result = serde_json::to_string(&info).unwrap();
                Response { success: true, result: Some(result), error: None }
            }
            Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
        }
    }

    async fn handle_backup_purge_old(&self, num_backups_to_keep: Option<usize>) -> Response {
        match self.backup_manager.purge_old_backups(num_backups_to_keep.unwrap_or(0)) {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
        }
    }

    async fn handle_backup_restore(&self, backup_id: Option<u32>, restore_path: Option<String>) -> Response {
        match (backup_id, restore_path) {
            (Some(backup_id), Some(restore_path)) => {
                match self.backup_manager.restore_from_backup(backup_id, restore_path) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
                }
            }
            _ => Response { success: false, result: None, error: Some("Missing backup ID or restore path".to_string()) }
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

    async fn handle_seek_to_first(&self) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::Start);
        if let Some(Ok((key, _))) = iter.next() {
            Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
        }
    }

    async fn handle_seek_to_last(&self) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::End);
        if let Some(Ok((key, _))) = iter.next() {
            Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
        }
    }

    async fn handle_seek(&self, key: String) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(key.as_bytes(), rust_rocksdb::Direction::Forward));
        if let Some(Ok((key, _))) = iter.next() {
            Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
        }
    }

    async fn handle_seek_for_prev(&self, key: String) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(key.as_bytes(), rust_rocksdb::Direction::Reverse));
        if let Some(Ok((key, _))) = iter.next() {
            Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
        }
    }

    async fn handle_valid(&self) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::Start);
        let valid = iter.next().is_some();
        Response { success: true, result: Some(valid.to_string()), error: None }
    }

    async fn handle_next(&self) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::Start);
        iter.next();
        if let Some(Ok((key, value))) = iter.next() {
            Response { success: true, result: Some(format!("{}:{}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(value.to_vec()).unwrap())), error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
        }
    }

    async fn handle_prev(&self) -> Response {
        let db = self.db_manager.db.lock().unwrap();
        let mut iter = db.iterator(rust_rocksdb::IteratorMode::End);
        if let Some(Ok((key, value))) = iter.next() {
            Response { success: true, result: Some(format!("{}:{}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(value.to_vec()).unwrap())), error: None }
        } else {
            Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <host> <db_path> <port> [ttl]", args[0]);
        return;
    }
    let db_path = &args[1];
    let host = &args[2];
    let port = &args[3];
    let ttl = if args.len() == 5 {
        Some(args[4].parse::<u64>().expect("Invalid TTL value"))
    } else {
        None
    };

    let addr = format!("{}:{}", host, port);

    let server = RocksDBServer::new(db_path.clone(), db_path.clone(), ttl).unwrap();

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on {}", addr);

    while let Ok((socket, _)) = listener.accept().await {
        let server = server.clone();
        let socket = Framed::new(socket, LengthDelimitedCodec::new());
        tokio::spawn(async move {
            server.handle_client(socket).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;
    use tokio_util::codec::{Framed, LengthDelimitedCodec};
    use bytes::BytesMut;
    use futures::SinkExt;
    use log::info;
    use crate::backup_manager::{BackupInfo};

    async fn send_request(socket: &mut Framed<TcpStream, LengthDelimitedCodec>, request: Request) -> Response {
        let request_bytes = serde_json::to_vec(&request).unwrap();
        socket.send(BytesMut::from(&request_bytes[..]).into()).await.unwrap();
        let response_bytes = socket.next().await.unwrap().unwrap();
        serde_json::from_slice(&response_bytes).unwrap()
    }

    #[tokio::test]
    async fn test_put_get() {
        let db_path = ".temp/test_db2";
        let backup_path = ".temp/test_backup";
        let server = RocksDBServer::new(db_path.to_string(), backup_path.to_string(), None).unwrap();
        let addr = "127.0.0.1:12345";
        let listener = TcpListener::bind(&addr).await.unwrap();
        let server_clone = server.clone();

        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                let server = server_clone.clone();
                let socket = Framed::new(socket, LengthDelimitedCodec::new());
                tokio::spawn(async move {
                    server.handle_client(socket).await;
                });
            }
        });

        let mut socket = Framed::new(TcpStream::connect(&addr).await.unwrap(), LengthDelimitedCodec::new());

        let put_request = Request {
            action: "put".to_string(),
            key: Some("test_key".to_string()),
            value: Some("test_value".to_string()),
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let put_response = send_request(&mut socket, put_request).await;
        assert!(put_response.success);

        let get_request = Request {
            action: "get".to_string(),
            key: Some("test_key".to_string()),
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let get_response = send_request(&mut socket, get_request).await;
        assert!(get_response.success);
        assert_eq!(get_response.result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_delete() {
        let db_path = ".temp/test_db1";
        let backup_path = ".temp/test_backup";
        let server = RocksDBServer::new(db_path.to_string(), backup_path.to_string(), None).unwrap();
        let addr = "127.0.0.1:12346";
        let listener = TcpListener::bind(&addr).await.unwrap();
        let server_clone = server.clone();

        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                let server = server_clone.clone();
                let socket = Framed::new(socket, LengthDelimitedCodec::new());
                tokio::spawn(async move {
                    server.handle_client(socket).await;
                });
            }
        });

        let mut socket = Framed::new(TcpStream::connect(&addr).await.unwrap(), LengthDelimitedCodec::new());

        let put_request = Request {
            action: "put".to_string(),
            key: Some("test_key".to_string()),
            value: Some("test_value".to_string()),
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        send_request(&mut socket, put_request).await;

        let delete_request = Request {
            action: "delete".to_string(),
            key: Some("test_key".to_string()),
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let delete_response = send_request(&mut socket, delete_request).await;
        assert!(delete_response.success);

        let get_request = Request {
            action: "get".to_string(),
            key: Some("test_key".to_string()),
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let get_response = send_request(&mut socket, get_request).await;
        assert!(!get_response.success);
    }

    #[tokio::test]
    async fn test_backup_restore() {
        let db_path = ".temp/test_db3";
        let backup_path = ".temp/test_backup";
        let server = RocksDBServer::new(db_path.to_string(), backup_path.to_string(), None).unwrap();
        let addr = "127.0.0.1:12345";
        let listener = TcpListener::bind(&addr).await.unwrap();
        let server_clone = server.clone();

        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                let server = server_clone.clone();
                let socket = Framed::new(socket, LengthDelimitedCodec::new());
                tokio::spawn(async move {
                    server.handle_client(socket).await;
                });
            }
        });

        let mut socket = Framed::new(TcpStream::connect(&addr).await.unwrap(), LengthDelimitedCodec::new());

        let put_request = Request {
            action: "put".to_string(),
            key: Some("test_key".to_string()),
            value: Some("test_value".to_string()),
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        send_request(&mut socket, put_request).await;

        let backup_request = Request {
            action: "backup_create".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let backup_response = send_request(&mut socket, backup_request).await;
        assert!(backup_response.success);

        let delete_request = Request {
            action: "delete".to_string(),
            key: Some("test_key".to_string()),
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        send_request(&mut socket, delete_request).await;

        let get_request = Request {
            action: "get".to_string(),
            key: Some("test_key".to_string()),
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let get_response = send_request(&mut socket, get_request).await;
        assert!(!get_response.success);

        let backup_info_request = Request {
            action: "backup_info".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let backup_info_response = send_request(&mut socket, backup_info_request).await;
        assert!(backup_info_response.success);
        let backup_info: Vec<BackupInfo> = serde_json::from_str(&backup_info_response.result.unwrap()).unwrap();
        assert!(!backup_info.is_empty());

        let restore_request = Request {
            action: "backup_restore".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: Some(backup_info[0].id),
            restore_path: Some(db_path.to_string()),
        };
        let restore_response = send_request(&mut socket, restore_request).await;
        assert!(restore_response.success);

        let get_request_after_restore = Request {
            action: "get".to_string(),
            key: Some("test_key".to_string()),
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let get_response_after_restore = send_request(&mut socket, get_request_after_restore).await;
        info!("{:?}",get_response_after_restore);
        assert!(get_response_after_restore.success);
        assert_eq!(get_response_after_restore.result, Some("test_value".to_string()));
    }
}
