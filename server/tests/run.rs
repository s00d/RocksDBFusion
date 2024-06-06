use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::BytesMut;
use futures::SinkExt;
use tokio_stream::StreamExt;

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

#[derive(Debug, Serialize, Deserialize)]
struct BackupInfo {
    id: u32,
    size: u64,
    num_files: u32,
    timestamp: i64,
}

async fn send_request(socket: &mut Framed<TcpStream, LengthDelimitedCodec>, request: Request) -> Response {
    let request_bytes = serde_json::to_vec(&request).unwrap();
    socket.send(BytesMut::from(&request_bytes[..]).into()).await.unwrap();
    let frame = socket.next().await.unwrap().unwrap();
    serde_json::from_slice(&frame).unwrap()
}

async fn setup_client() -> Framed<TcpStream, LengthDelimitedCodec> {
    let addr = "127.0.0.1:12345";
    let stream = TcpStream::connect(addr).await.unwrap();
    Framed::new(stream, LengthDelimitedCodec::new())
}

#[tokio::test]
async fn test_put_get() {
    let mut client = setup_client().await;

    // Test put
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
    let put_response = send_request(&mut client, put_request).await;
    assert!(put_response.success);

    // Test get
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
    let get_response = send_request(&mut client, get_request).await;
    assert!(get_response.success);
    assert_eq!(get_response.result, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_delete() {
    let mut client = setup_client().await;

    // Test delete
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
    let delete_response = send_request(&mut client, delete_request).await;
    assert!(delete_response.success);

    // Verify deletion
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
    let get_response = send_request(&mut client, get_request).await;
    assert!(!get_response.success);
}

#[tokio::test]
async fn test_merge() {
    let mut client = setup_client().await;

    // Test merge
    let merge_request = Request {
        action: "merge".to_string(),
        key: Some("merge_key".to_string()),
        value: Some("{\"a\":1}".to_string()),
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let merge_response = send_request(&mut client, merge_request).await;
    assert!(merge_response.success);

    // Verify merge
    let get_request = Request {
        action: "get".to_string(),
        key: Some("merge_key".to_string()),
        value: None,
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let get_response = send_request(&mut client, get_request).await;
    assert!(get_response.success);
    assert_eq!(get_response.result, Some("{\"a\":1}".to_string()));
}

#[tokio::test]
async fn test_transaction() {
    let mut client = setup_client().await;

    // Begin transaction
    let begin_txn_request = Request {
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
    let begin_txn_response = send_request(&mut client, begin_txn_request).await;
    assert!(begin_txn_response.success);

    // Put within transaction
    let put_request = Request {
        action: "put".to_string(),
        key: Some("txn_key".to_string()),
        value: Some("txn_value".to_string()),
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let put_response = send_request(&mut client, put_request).await;
    assert!(put_response.success);

    // Commit transaction
    let commit_txn_request = Request {
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
    let commit_txn_response = send_request(&mut client, commit_txn_request).await;
    assert!(commit_txn_response.success);

    // Verify commit
    let get_request = Request {
        action: "get".to_string(),
        key: Some("txn_key".to_string()),
        value: None,
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let get_response = send_request(&mut client, get_request).await;
    assert!(get_response.success);
    assert_eq!(get_response.result, Some("txn_value".to_string()));
}

#[tokio::test]
async fn test_backup_restore() {
    let mut client = setup_client().await;

    // Test put before backup
    let put_request = Request {
        action: "put".to_string(),
        key: Some("backup_key".to_string()),
        value: Some("backup_value".to_string()),
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let put_response = send_request(&mut client, put_request).await;
    assert!(put_response.success);

    // Create backup
    let backup_create_request = Request {
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
    let backup_create_response = send_request(&mut client, backup_create_request).await;
    assert!(backup_create_response.success);

    // Delete key to test restore
    let delete_request = Request {
        action: "delete".to_string(),
        key: Some("backup_key".to_string()),
        value: None,
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let delete_response = send_request(&mut client, delete_request).await;
    assert!(delete_response.success);

    // Restore backup
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
    let backup_info_response = send_request(&mut client, backup_info_request).await;
    let backup_info: Vec<BackupInfo> = serde_json::from_str(backup_info_response.result.as_ref().unwrap()).unwrap();
    let latest_backup_id = backup_info.last().unwrap().id;

    let backup_restore_request = Request {
        action: "backup_restore".to_string(),
        key: None,
        value: None,
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: Some(latest_backup_id),
        restore_path: Some("path_to_restore_db".to_string()),
    };
    let backup_restore_response = send_request(&mut client, backup_restore_request).await;
    assert!(backup_restore_response.success);

    // Verify restore
    let get_request = Request {
        action: "get".to_string(),
        key: Some("backup_key".to_string()),
        value: None,
        cf_name: None,
        options: None,
        backup_path: None,
        num_backups_to_keep: None,
        backup_id: None,
        restore_path: None,
    };
    let get_response = send_request(&mut client, get_request).await;
    assert!(get_response.success);
    assert_eq!(get_response.result, Some("backup_value".to_string()));
}
