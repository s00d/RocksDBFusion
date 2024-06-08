use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;
use ext_php_rs::prelude::PhpResult;
use futures::StreamExt;
use futures::SinkExt;

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
    iterator_id: Option<usize>,
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

    async fn get_connection(&self) -> Result<Arc<Mutex<Framed<TcpStream, LengthDelimitedCodec>>>, String> {
        let mut conn = self.connection.lock().await;
        if conn.is_none() {
            let addr = format!("{}:{}", self.host, self.port);
            let stream = TcpStream::connect(&addr).await.map_err(|e| format!("Connection error: {}", e))?;
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

        let request_bytes = serde_json::to_vec(&request).map_err(|e| format!("Serialization error: {}", e))?;
        conn.send(Bytes::from(request_bytes)).await.map_err(|e| format!("Send error: {}", e))?;

        let response_bytes = match conn.next().await {
            Some(Ok(bytes)) => bytes,
            Some(Err(e)) => return Err(format!("Receive error: {}", e)),
            None => return Err("Receive error: no response received".to_string()),
        };

        let response: Response = serde_json::from_slice(&response_bytes).map_err(|e| format!("Deserialization error: {}", e))?;
        Ok(response)
    }

    pub fn handle_response(&self, response: Response) -> PhpResult<Option<String>> {
        if response.success {
            Ok(response.result)
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
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
                cf_name: None,
                options: None,
                backup_path: None,
                num_backups_to_keep: None,
                backup_id: None,
                restore_path: None,
                iterator_id: None,
            },
        }
    }

    pub fn set_key(mut self, key: String) -> Self {
        self.request.key = Some(key);
        self
    }

    pub fn set_value(mut self, value: String) -> Self {
        self.request.value = Some(value);
        self
    }

    pub fn set_cf_name(mut self, cf_name: String) -> Self {
        self.request.cf_name = Some(cf_name);
        self
    }

    // pub fn set_options(mut self, options: HashMap<String, String>) -> Self {
    //     self.request.options = Some(options);
    //     self
    // }
    //
    // pub fn set_backup_path(mut self, backup_path: String) -> Self {
    //     self.request.backup_path = Some(backup_path);
    //     self
    // }

    pub fn set_num_backups_to_keep(mut self, num_backups_to_keep: usize) -> Self {
        self.request.num_backups_to_keep = Some(num_backups_to_keep);
        self
    }

    pub fn set_backup_id(mut self, backup_id: u32) -> Self {
        self.request.backup_id = Some(backup_id);
        self
    }

    pub fn set_restore_path(mut self, restore_path: String) -> Self {
        self.request.restore_path = Some(restore_path);
        self
    }

    pub fn set_iterator_id(mut self, iterator_id: usize) -> Self {
        self.request.iterator_id = Some(iterator_id);
        self
    }

    pub fn build(self) -> Request {
        self.request
    }
}