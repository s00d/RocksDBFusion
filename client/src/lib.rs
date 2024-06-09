#![cfg_attr(all(windows, target_arch = "x86_64"), feature(abi_vectorcall))]
#![no_main]

mod request_builder;

use crate::request_builder::{RequestBuilder, RequestHandler};
use ext_php_rs::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

#[php_class(name = "RocksDBClient")]
pub struct RocksDBClient {
    request_handler: RequestHandler,
}

#[php_impl(rename_methods = "camelCase")]
impl RocksDBClient {
    #[constructor]
    pub fn __construct(host: String, port: u16) -> PhpResult<Self> {
        Ok(Self {
            request_handler: RequestHandler::new(host, port),
        })
    }

    #[php_method]
    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = RequestBuilder::new("put")
            .set_key(key)
            .set_value(value)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn get(&self, key: String, cf_name: Option<String>) -> PhpResult<Option<String>> {
        let request = RequestBuilder::new("get")
            .set_key(key)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response)
    }

    #[php_method]
    pub fn delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = RequestBuilder::new("delete")
            .set_key(key)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = RequestBuilder::new("merge")
            .set_key(key)
            .set_value(value)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn list_column_families(&self, path: String) -> PhpResult<Vec<String>> {
        let request = RequestBuilder::new("list_column_families")
            .set_value(path)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        if response.success {
            let result: Vec<String> =
                serde_json::from_str(&response.result.unwrap_or("[]".to_string()))
                    .map_err(|e| format!("Deserialization error: {}", e))?;
            Ok(result)
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn create_column_family(&self, cf_name: String) -> PhpResult<()> {
        let request = RequestBuilder::new("create_column_family")
            .set_value(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn drop_column_family(&self, cf_name: String) -> PhpResult<()> {
        let request = RequestBuilder::new("drop_column_family")
            .set_value(cf_name)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn compact_range(
        &self,
        start: Option<String>,
        end: Option<String>,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
        let mut request_builder = RequestBuilder::new("compact_range");

        if let Some(start_key) = start {
            request_builder = request_builder.set_key(start_key);
        }

        if let Some(end_key) = end {
            request_builder = request_builder.set_value(end_key);
        }

        if let Some(cf) = cf_name {
            request_builder = request_builder.set_cf_name(cf);
        }

        let request = request_builder.build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    // -- write batch methods
    #[php_method]
    pub fn write_batch_put(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
        let request = RequestBuilder::new("write_batch_put")
            .set_key(key)
            .set_value(value)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn write_batch_merge(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
        let request = RequestBuilder::new("write_batch_merge")
            .set_key(key)
            .set_value(value)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn write_batch_delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        let request = RequestBuilder::new("write_batch_delete")
            .set_key(key)
            .set_cf_name(cf_name.unwrap_or_default())
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn write_batch_write(&self) -> PhpResult<()> {
        let request = RequestBuilder::new("write_batch_write").build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn write_batch_clear(&self) -> PhpResult<()> {
        let request = RequestBuilder::new("write_batch_clear").build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    #[php_method]
    pub fn write_batch_destroy(&self) -> PhpResult<()> {
        let request = RequestBuilder::new("write_batch_destroy").build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response).map(|_| ())
    }

    // -- iterator methods
    #[php_method]
    pub fn create_iterator(&self) -> PhpResult<Option<String>> {
        let request = RequestBuilder::new("create_iterator").build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response)
    }

    #[php_method]
    pub fn destroy_iterator(&self, iterator_id: usize) -> PhpResult<Option<String>> {
        let request = RequestBuilder::new("destroy_iterator")
            .set_iterator_id(iterator_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response)
    }

    #[php_method]
    pub fn iterator_seek(&self, iterator_id: usize, key: String) -> PhpResult<Option<String>> {
        let request = RequestBuilder::new("iterator_seek")
            .set_iterator_id(iterator_id)
            .set_key(key)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response)
    }

    #[php_method]
    pub fn iterator_seek_for_prev(
        &self,
        iterator_id: usize,
        key: String,
    ) -> PhpResult<Option<String>> {
        let request = RequestBuilder::new("iterator_seek_for_prev")
            .set_iterator_id(iterator_id)
            .set_key(key)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response)
    }

    #[php_method]
    pub fn iterator_next(&self, iterator_id: usize) -> PhpResult<bool> {
        let request = RequestBuilder::new("iterator_next")
            .set_iterator_id(iterator_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        if response.success {
            Ok(response.result.unwrap().parse().unwrap_or(false))
        } else {
            Err(response.error.unwrap_or("Unknown error".to_string()).into())
        }
    }

    #[php_method]
    pub fn iterator_prev(&self, iterator_id: usize) -> PhpResult<Option<String>> {
        let request = RequestBuilder::new("iterator_prev")
            .set_iterator_id(iterator_id)
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| PhpException::default(format!("Error sending request: {}", e)))?;

        self.request_handler.handle_response(response)
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
