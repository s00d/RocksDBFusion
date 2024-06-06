use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::BytesMut;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Request {
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
struct Response {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

#[php_class(name = "RocksDBClient")]
pub struct RocksDBClient {
    host: String,
    port: u16,
}

#[php_impl]
impl RocksDBClient {
    #[constructor]
    pub fn __construct(host: String, port: u16) -> PhpResult<Self> {
        Ok(Self { host, port })
    }

    async fn send_request(&self, request: Request) -> PhpResult<Response> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(&addr).await.map_err(|e| format!("Connection error: {}", e))?;
        let mut socket = Framed::new(stream, LengthDelimitedCodec::new());
        let request_bytes = serde_json::to_vec(&request).map_err(|e| format!("Serialization error: {}", e))?;
        socket.send(BytesMut::from(&request_bytes[..])).await.map_err(|e| format!("Send error: {}", e))?;
        let response_bytes = socket.next().await.ok_or("Receive error")??;
        let response: Response = serde_json::from_slice(&response_bytes).map_err(|e| format!("Deserialization error: {}", e))?;
        Ok(response)
    }

    #[php_method]
    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = Request {
            action: "put".to_string(),
            key: Some(key),
            value: Some(value),
            cf_name,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn get(&self, key: String, cf_name: Option<String>) -> PhpResult<Option<String>> {
        let request = Request {
            action: "get".to_string(),
            key: Some(key),
            value: None,
            cf_name,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(response.result)
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = Request {
            action: "delete".to_string(),
            key: Some(key),
            value: None,
            cf_name,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = Request {
            action: "merge".to_string(),
            key: Some(key),
            value: Some(value),
            cf_name,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn list_column_families(&self, path: String) -> PhpResult<Vec<String>> {
        let request = Request {
            action: "list_column_families".to_string(),
            key: None,
            value: Some(path),
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            let result: Vec<String> = serde_json::from_str(&response.result.unwrap_or("[]".to_string()))
                .map_err(|e| format!("Deserialization error: {}", e))?;
            Ok(result)
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn create_column_family(&self, cf_name: String) -> PhpResult<()> {
        let request = Request {
            action: "create_column_family".to_string(),
            key: None,
            value: Some(cf_name),
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn drop_column_family(&self, cf_name: String) -> PhpResult<()> {
        let request = Request {
            action: "drop_column_family".to_string(),
            key: None,
            value: Some(cf_name),
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn compact_range(&self, start: Option<String>, end: Option<String>, cf_name: Option<String>) -> PhpResult<()> {
        let request = Request {
            action: "compact_range".to_string(),
            key: start,
            value: end,
            cf_name,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    // -- transaction methods
    #[php_method]
    pub fn begin_transaction(&self) -> PhpResult<()> {
        let request = Request {
            action: "begin_transaction".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn commit_transaction(&self) -> PhpResult<()> {
        let request = Request {
            action: "commit_transaction".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn rollback_transaction(&self) -> PhpResult<()> {
        let request = Request {
            action: "rollback_transaction".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn set_savepoint(&self) -> PhpResult<()> {
        let request = Request {
            action: "set_savepoint".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn rollback_to_savepoint(&self) -> PhpResult<()> {
        let request = Request {
            action: "rollback_to_savepoint".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    // -- backup methods
    #[php_method]
    pub fn backup_create(&self) -> PhpResult<()> {
        let request = Request {
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
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn backup_info(&self) -> PhpResult<HashMap<String, i64>> {
        let request = Request {
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
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            let result: HashMap<String, i64> = serde_json::from_str(&response.result.unwrap_or("{}".to_string()))
                .map_err(|e| format!("Deserialization error: {}", e))?;
            Ok(result)
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn backup_purge_old(&self, num_backups_to_keep: usize) -> PhpResult<()> {
        let request = Request {
            action: "backup_purge_old".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: Some(num_backups_to_keep),
            backup_id: None,
            restore_path: None,
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn backup_restore(&self, backup_id: u32, restore_path: String) -> PhpResult<()> {
        let request = Request {
            action: "backup_restore".to_string(),
            key: None,
            value: None,
            cf_name: None,
            options: None,
            backup_path: None,
            num_backups_to_keep: None,
            backup_id: Some(backup_id),
            restore_path: Some(restore_path),
        };
        let response = Runtime::new()
            .unwrap()
            .block_on(self.send_request(request))?;

        if response.success {
            Ok(())
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}