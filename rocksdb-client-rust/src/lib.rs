use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use serde::{Deserialize, Serialize};

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
    txn: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub result: Option<String>,
}

pub struct RequestHandler {
    host: String,
    port: u16,
    connection: Option<TcpStream>,
}

impl RequestHandler {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            connection: None,
        }
    }

    fn get_connection(&mut self) -> Result<&mut TcpStream, String> {
        if self.connection.is_none() {
            let addr = format!("{}:{}", self.host, self.port);
            let stream = TcpStream::connect(&addr).map_err(|e| format!("Connection error: {}", e))?;
            self.connection = Some(stream);
        }
        self.connection.as_mut().ok_or_else(|| "Failed to acquire connection".to_string())
    }

    pub fn send_request(&mut self, request: Request) -> Result<Response, String> {
        let conn = self.get_connection()?;

        let request_bytes = serde_json::to_vec(&request).map_err(|e| format!("Serialization error: {}", e))?;
        conn.write_all(&request_bytes).map_err(|e| format!("Send error: {}", e))?;
        conn.write_all(b"\n").map_err(|e| format!("Send error: {}", e))?;

        let mut reader = BufReader::new(conn);
        let mut response_bytes = Vec::new();
        reader.read_until(b'\n', &mut response_bytes).map_err(|e| format!("Receive error: {}", e))?;

        let response: Response = serde_json::from_slice(&response_bytes).map_err(|e| format!("Deserialization error: {}", e))?;
        Ok(response)
    }

    pub fn handle_response(&self, response: Response) -> Result<Option<String>, String> {
        if response.success {
            Ok(response.result)
        } else {
            Err(response.result.unwrap_or("Unknown error".to_string()))
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
                txn: None,
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

    pub fn txn(mut self, txn: Option<bool>) -> Self {
        self.request.txn = txn;
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

    pub fn put(&mut self, key: String, value: String, cf_name: Option<String>, txn: Option<bool>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("put")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .txn(txn)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn get(&mut self, key: String, cf_name: Option<String>, default_value: Option<String>, txn: Option<bool>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("get")
            .key(Some(key))
            .cf_name(cf_name)
            .default_value(default_value)
            .txn(txn)
            .build();


        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn delete(&mut self, key: String, cf_name: Option<String>, txn: Option<bool>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("delete")
            .key(Some(key))
            .cf_name(cf_name)
            .txn(txn)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn merge(&mut self, key: String, value: String, cf_name: Option<String>, txn: Option<bool>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("merge")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .txn(txn)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn get_property(&mut self, value: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("get_property")
            .value(Some(value))
            .cf_name(cf_name)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn keys(&mut self, start: String, limit: String, query: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("keys")
            .option("start".to_string(), start)
            .option("limit".to_string(), limit)
            .option("query".to_string(), query.unwrap_or_default())
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn all(&mut self, query: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("all")
            .option("query".to_string(), query.unwrap_or_default())
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn list_column_families(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("list_column_families")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn create_column_family(&mut self, cf_name: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("create_column_family")
            .cf_name(Some(cf_name))
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn drop_column_family(&mut self, cf_name: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("drop_column_family")
            .cf_name(Some(cf_name))
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn compact_range(&mut self, start: Option<String>, end: Option<String>, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("compact_range")
            .option("start".to_string(), start.unwrap_or_default())
            .option("end".to_string(), end.unwrap_or_default())
            .cf_name(cf_name)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn write_batch_put(&mut self, key: String, value: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_put")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn write_batch_merge(&mut self, key: String, value: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_merge")
            .key(Some(key))
            .value(Some(value))
            .cf_name(cf_name)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn write_batch_delete(&mut self, key: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_delete")
            .key(Some(key))
            .cf_name(cf_name)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn write_batch_write(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_write")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn write_batch_clear(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_clear")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn write_batch_destroy(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("write_batch_destroy")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn create_iterator(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("create_iterator")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn destroy_iterator(&mut self, iterator_id: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("destroy_iterator")
            .option("iterator_id".to_string(), iterator_id)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn iterator_seek(&mut self, iterator_id: String, key: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("iterator_seek")
            .option("iterator_id".to_string(), iterator_id)
            .key(Some(key))
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn iterator_next(&mut self, iterator_id: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("iterator_next")
            .option("iterator_id".to_string(), iterator_id)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn iterator_prev(&mut self, iterator_id: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("iterator_prev")
            .option("iterator_id".to_string(), iterator_id)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn backup(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("backup")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn restore_latest(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("restore_latest")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn restore(&mut self, backup_id: String) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("restore")
            .option("backup_id".to_string(), backup_id)
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn get_backup_info(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("get_backup_info")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn begin_transaction(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("begin_transaction")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn commit_transaction(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("commit_transaction")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }

    pub fn rollback_transaction(&mut self) -> Result<Option<String>, String> {
        let request = RequestBuilder::new("rollback_transaction")
            .build();

        let response = self.request_handler.send_request(request)?;
        self.request_handler.handle_response(response)
    }
}
