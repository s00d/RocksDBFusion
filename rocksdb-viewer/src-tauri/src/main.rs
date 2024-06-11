use std::collections::HashMap;
use std::io::{Write, BufRead, BufReader};
use std::net::{TcpStream};
use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tokio::sync::Mutex as AsyncMutex;

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    action: String,
    key: Option<String>,
    value: Option<String>,
    default: Option<String>,
    cf_name: Option<String>,
    options: Option<HashMap<String, String>>,
    iterator_id: Option<usize>,
    token: Option<String>,
    ssh_info: Option<[String; 4]>, // [host, user, password, port]
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

struct ServerState {
    address: Option<String>,
    token: Option<String>,
    ssh_info: Option<[String; 4]>,
}

impl ServerState {
    async fn send_request(&self, request: Request) -> Result<Response, String> {
        let address = self.address.clone().ok_or("Server address not set")?;
        let mut stream = TcpStream::connect(&address).map_err(|e| e.to_string())?;
        let request_json = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        stream.write_all(request_json.as_bytes()).map_err(|e| e.to_string())?;
        stream.write_all(b"\n").map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream);
        let mut response_json = String::new();
        reader.read_line(&mut response_json).map_err(|e| e.to_string())?;
        serde_json::from_str(&response_json).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn connect_to_server(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    address: String,
    token: Option<String>,
    ssh_info: Option<[String; 4]>
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.address = Some(address.clone());
    state.token = token;
    state.ssh_info = ssh_info;

    // Test the connection with a simple request
    let test_request = Request {
        action: "test".to_string(),
        key: None,
        value: None,
        default: None,
        cf_name: None,
        options: None,
        iterator_id: None,
        token: state.token.clone(),
        ssh_info: state.ssh_info.clone(),
    };

    match state.send_request(test_request).await {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to connect to server: {}", err)),
    }
}

#[tauri::command]
async fn get_keys(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    start: usize,
    limit: usize,
    query: Option<String>
) -> Result<Vec<String>, String> {
    let state = state.lock().await;
    let request = Request {
        action: "keys".to_string(),
        key: None,
        value: None,
        default: None,
        cf_name: None,
        options: Some(vec![
            ("start".to_string(), start.to_string()),
            ("limit".to_string(), limit.to_string()),
            ("query".to_string(), query.unwrap_or_default())
        ].into_iter().collect()),
        iterator_id: None,
        token: state.token.clone(),
        ssh_info: state.ssh_info.clone(),
    };
    let response = state.send_request(request).await?;
    response.result
        .ok_or("Failed to get keys".to_string())
        .and_then(|res| serde_json::from_str(&res).map_err(|e| e.to_string()))
}

#[tauri::command]
async fn get_value(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    key: String
) -> Result<String, String> {
    let state = state.lock().await;
    let request = Request {
        action: "get".to_string(),
        key: Some(key),
        value: None,
        default: None,
        cf_name: None,
        options: None,
        iterator_id: None,
        token: state.token.clone(),
        ssh_info: state.ssh_info.clone(),
    };
    let response = state.send_request(request).await?;
    response.result.ok_or("Failed to get value".to_string())
}

#[tauri::command]
async fn put_value(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    key: String,
    value: String
) -> Result<(), String> {
    let state = state.lock().await;
    let request = Request {
        action: "put".to_string(),
        key: Some(key),
        value: Some(value),
        default: None,
        cf_name: None,
        options: None,
        iterator_id: None,
        token: state.token.clone(),
        ssh_info: state.ssh_info.clone(),
    };
    let response = state.send_request(request).await?;
    if response.success {
        Ok(())
    } else {
        Err(response.error.unwrap_or("Failed to put value".to_string()))
    }
}

#[tauri::command]
async fn delete_value(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    key: String
) -> Result<(), String> {
    let state = state.lock().await;
    let request = Request {
        action: "delete".to_string(),
        key: Some(key),
        value: None,
        default: None,
        cf_name: None,
        options: None,
        iterator_id: None,
        token: state.token.clone(),
        ssh_info: state.ssh_info.clone(),
    };
    let response = state.send_request(request).await?;
    if response.success {
        Ok(())
    } else {
        Err(response.error.unwrap_or("Failed to delete value".to_string()))
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let state = Arc::new(AsyncMutex::new(ServerState {
                address: None,
                token: None,
                ssh_info: None,
            }));
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_to_server,
            get_keys,
            get_value,
            put_value,
            delete_value
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
