use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex as AsyncMutex;
use rocksdb_client_rust::RocksDBClient;

struct ServerState {
    client: Option<RocksDBClient>,
    token: Option<String>,
    ssh_info: Option<[String; 4]>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            client: None,
            token: None,
            ssh_info: None,
        }
    }
}

#[tauri::command]
async fn connect_to_server(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    host: String,
    port: u16,
    token: Option<String>,
    ssh_info: Option<[String; 4]>
) -> Result<(), String> {
    let mut state = state.lock().await;
    println!("connecting: {}:{}", host.clone(), port);
    state.client = Some(RocksDBClient::new(host.clone(), port));
    state.token = token;
    state.ssh_info = ssh_info;

    // Test the connection with a simple request
    match state.client.as_mut().unwrap().list_column_families() {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("Failed to connect to server: {}", err);
            Err(format!("Failed to connect to server: {}", err))
        },
    }
}

#[tauri::command]
async fn get_keys(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    start: usize,
    limit: usize,
    query: Option<String>
) -> Result<Vec<String>, String> {
    let mut state = state.lock().await;
    let client = state.client.as_mut().ok_or("Client not initialized")?;

    let keys_json = client.keys(start.to_string(), limit.to_string(), query).map_err(|e| e.to_string())?;
    let keys: Vec<String> = match keys_json {
        Some(json_str) => serde_json::from_str(&json_str).map_err(|e| e.to_string())?,
        None => Vec::new(),
    };
    Ok(keys)
}

#[tauri::command]
async fn get_value(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    key: String
) -> Result<String, String> {
    let mut state = state.lock().await;
    let client = state.client.as_mut().ok_or("Client not initialized")?;

    client.get(key, None, None, None)
        .map_err(|e| e.to_string())
        .and_then(|res| res.ok_or("Key not found".to_string()))
}

#[tauri::command]
async fn put_value(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    key: String,
    value: String
) -> Result<(), String> {
    let mut state = state.lock().await;
    let client = state.client.as_mut().ok_or("Client not initialized")?;

    client.put(key, value, None, None)
        .map_err(|e| e.to_string())
        .and_then(|res| res.ok_or("Failed to put value".to_string()).map(|_| ()))
}

#[tauri::command]
async fn delete_value(
    state: tauri::State<'_, Arc<AsyncMutex<ServerState>>>,
    key: String
) -> Result<(), String> {
    let mut state = state.lock().await;
    let client = state.client.as_mut().ok_or("Client not initialized")?;

    client.delete(key, None, None)
        .map_err(|e| e.to_string())
        .and_then(|res| res.ok_or("Failed to delete value".to_string()).map(|_| ()))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let state = Arc::new(AsyncMutex::new(ServerState::new()));
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
