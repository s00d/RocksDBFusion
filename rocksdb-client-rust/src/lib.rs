
use std::collections::HashMap;
use tokio::runtime::Runtime;
use bytes::Bytes;
use futures::SinkExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    action: String,
    key: Option<String>,
    value: Option<String>,
    cf_name: Option<String>,
    default_value: Option<String>,
    options: Option<HashMap<String, String>>,
    backup_path: Option<String>,
    num_backups_to_keep: Option<usize>,
    backup_id: Option<u32>,
    restore_path: Option<String>,
    iterator_id: Option<usize>,
    txn_id: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

pub struct RequestHandler {
    host: String,
    port: u16,
    connection: Arc<Mutex<Option<Framed<TcpStream, LengthDelimitedCodec>>>>,
}

impl RequestHandler {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            connection: Arc::new(Mutex::new(None)),
        }
    }

    async fn get_connection(
        &self,
    ) -> Result<Arc<Mutex<Framed<TcpStream, LengthDelimitedCodec>>>, String> {
        let mut conn = self.connection.lock().await;
        if conn.is_none() {
            let addr = format!("{}:{}", self.host, self.port);
            let stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| format!("Connection error: {}", e))?;
            let framed = Framed::new(stream, LengthDelimitedCodec::new());
            *conn = Some(framed);
        }

        if let Some(framed) = conn.take() {
            Ok(Arc::new(Mutex::new(framed)))
        } else {
            Err("Failed to acquire connection".to_string())
        }
    }

    pub async fn send_request(&self, request: Request) -> Result<Response, String> {
        let connection = self.get_connection().await?;
        let mut conn = connection.lock().await;

        let request_bytes =
            serde_json::to_vec(&request).map_err(|e| format!("Serialization error: {}", e))?;
        conn.send(Bytes::from(request_bytes))
            .await
            .map_err(|e| format!("Send error: {}", e))?;

        let response_bytes = match conn.next().await {
            Some(Ok(bytes)) => bytes,
            Some(Err(e)) => return Err(format!("Receive error: {}", e)),
            None => return Err("Receive error: no response received".to_string()),
        };

        let response: Response = serde_json::from_slice(&response_bytes)
            .map_err(|e| format!("Deserialization error: {}", e))?;
        Ok(response)
    }

    pub fn handle_response(&self, response: Response) -> Result<Option<String>, String> {
        if response.success {
            Ok(response.result)
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    pub fn new(action: &str) -> Self {
        Self {
            request: Request {
                action: action.to_string(),
                key: None,
                value: None,
                default_value: None,
                cf_name: None,
                options: None,
                backup_path: None,
                num_backups_to_keep: None,
                backup_id: None,
                restore_path: None,
                iterator_id: None,
                txn_id: None,
            },
        }
    }

    pub fn key(mut self, key: Option<String>) -> Self {
        self.request.key = key;
        self
    }

    pub fn value(mut self, value: Option<String>) -> Self {
        self.request.value = value;
        self
    }
    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.request.default_value = value;
        self
    }

    pub fn cf_name(mut self, cf_name: Option<String>) -> Self {
        self.request.cf_name = cf_name;
        self
    }

    pub fn num_backups_to_keep(mut self, num_backups_to_keep: Option<usize>) -> Self {
        self.request.num_backups_to_keep = num_backups_to_keep;
        self
    }

    pub fn backup_id(mut self, backup_id: Option<u32>) -> Self {
        self.request.backup_id = backup_id;
        self
    }

    pub fn restore_path(mut self, restore_path: Option<String>) -> Self {
        self.request.restore_path = restore_path;
        self
    }

    pub fn iterator_id(mut self, iterator_id: Option<usize>) -> Self {
        self.request.iterator_id = iterator_id;
        self
    }
    
    pub fn txn_id(mut self, txn_id: Option<usize>) -> Self {
        self.request.txn_id = txn_id;
        self
    }

    pub fn option(mut self, key: String, value: String) -> Self {
        if self.request.options.is_none() {
            self.request.options = Some(HashMap::new());
        }
        self.request.options.as_mut().unwrap().insert(key, value);
        self
    }

    pub fn build(self) -> Request {
        self.request
    }
}

pub struct RocksDBClient {
    request_handler: RequestHandler,
}

impl RocksDBClient {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            request_handler: RequestHandler::new(host, port),
        }
    }

    
    pub fn put(&self, key: String, value: String, cf_name: Option<String>, txn_id: Option<usize>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("put")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .txn_id(txn_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn get(&self, key: String, cf_name: Option<String>, default_value: Option<String>, txn_id: Option<usize>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("get")
            .key(Some(key))
            .cf_name(cf_name)
            .default_value(default_value)
            .txn_id(txn_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn delete(&self, key: String, cf_name: Option<String>, txn_id: Option<usize>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("delete")
            .key(Some(key))
            .cf_name(cf_name)
            .txn_id(txn_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn merge(&self, key: String, value: String, cf_name: Option<String>, txn_id: Option<usize>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("merge")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .txn_id(txn_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn get_property(&self, value: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("get_property")
            .value(Some(value))
            .cf_name(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn keys(&self, start: usize, limit: usize, query: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("keys")
                .option("start".to_string(), start.to_string())
                .option("limit".to_string(), limit.to_string())
                .option("query".to_string(), query.unwrap().to_string())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn all(&self, query: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("all")
                .option("query".to_string(), query.unwrap().to_string())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn list_column_families(&self, value: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("list_column_families")
            .value(Some(value))
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn create_column_family(&self, cf_name: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("create_column_family")
            .cf_name(Some(cf_name))
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn drop_column_family(&self, cf_name: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("drop_column_family")
            .cf_name(Some(cf_name))
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn compact_range(&self, start: Option<String>, end: Option<String>, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("compact_range")
                .option("start".to_string(), start.unwrap().to_string())
                .option("end".to_string(), end.unwrap().to_string())
            .cf_name(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn write_batch_put(&self, key: String, value: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_put")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn write_batch_merge(&self, key: String, value: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_merge")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn write_batch_delete(&self, key: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_delete")
            .key(Some(key))
            .cf_name(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn write_batch_write(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_write")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn write_batch_clear(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_clear")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn write_batch_destroy(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_destroy")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn create_iterator(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("create_iterator")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn destroy_iterator(&self, iterator_id: usize) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("destroy_iterator")
                .option("iterator_id".to_string(), iterator_id.to_string())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn iterator_seek(&self, iterator_id: usize, key: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("iterator_seek")
                .option("iterator_id".to_string(), iterator_id.to_string())
            .key(Some(key))
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn iterator_next(&self, iterator_id: usize) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("iterator_next")
                .option("iterator_id".to_string(), iterator_id.to_string())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn iterator_prev(&self, iterator_id: usize) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("iterator_prev")
                .option("iterator_id".to_string(), iterator_id.to_string())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn backup(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("backup")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn restore_latest(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("restore_latest")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn restore(&self, backup_id: u32) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("restore")
                .option("backup_id".to_string(), backup_id.to_string())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn get_backup_info(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("get_backup_info")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn begin_transaction(&self, ) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("begin_transaction")
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn commit_transaction(&self, txn_id: usize) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("commit_transaction")
            .txn_id(Some(txn_id))
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }


    pub fn rollback_transaction(&self, txn_id: usize) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("rollback_transaction")
            .txn_id(Some(txn_id))
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }

}
