const fs = require('fs');
const Handlebars = require('handlebars');
const _ = require('lodash');


const typeConversion = {
    usize: 'usize',
    u32: 'u32',
    bool: 'bool',
    string: 'String',
};

const convertType = (type) => {
    return typeConversion[type] || type;
};

const convertNewlines = (text) => {
    return text ? text.replace(/\\n/g, '\n     * ') : '';
};

const processParameters = (params, parentKey = '') => {
    const result = [];
    for (const key in params) {
        const param = params[key];
        const paramName = parentKey ? `${parentKey}.${key}` : key;

        if (param.param_type === 'object' && param.properties) {
            result.push(...processParameters(param.properties, paramName));
        } else {
            result.push({
                name: paramName,
                param_type: convertType(param.param_type.toLowerCase()),
                description: param.description,
                required: param.required,
                notRequired: !param.required,
                inOptions: parentKey === 'options'
            });
        }
    }
    return result;
};

const methodTemplate = `
    pub fn {{actionSnake}}(&self, {{{parametersList}}}) -> Result<{{{returnType}}}, String> {
        let request = RequestBuilder::new("{{action}}")
            {{#each parameters}}
            {{#if inOptions}}
                {{#if required}}
                .option("{{replace name 'options.' ''}}".to_string(), {{replace name 'options.' ''}}.to_string())
                {{else}}
                .option("{{replace name 'options.' ''}}".to_string(), {{replace name 'options.' ''}}.unwrap().to_string())
                {{/if}}
            {{else}}
            {{#if notRequired}}
            .{{replace name 'options.' ''}}({{replace name 'options.' ''}})
            {{else}}
            .{{replace name 'options.' ''}}(Some({{replace name 'options.' ''}}))
            {{/if}}
            {{/if}}
            {{/each}}
            .build();

        let response = Runtime::new()
            .unwrap()
            .block_on(self.request_handler.send_request(request))
            .map_err(|e| format!("Error sending request: {}", e))?;

        self.request_handler.handle_response(response)
    }
`;

Handlebars.registerHelper('replace', (haystack, needle, replacement) => haystack.replace(needle, replacement));
Handlebars.registerHelper('snake_case', (str) => _.snakeCase(str));

const data = JSON.parse(fs.readFileSync(__dirname + '/../requests_schema.json', 'utf8'));

const generateMethods = (requests) => {
    return requests.map(request => {
        const allParameters = processParameters(request.parameters);

        const parametersList = allParameters.map(param => {
            const nullValue = param.required ? param.param_type : `Option<${param.param_type}>`;
            return `${param.name.replace('options.', '')}: ${nullValue}`;
        }).join(', ');

        const returnType = 'Option<String>';
        const returnResult = returnType !== '()';

        const methodData = {
            action: request.action,
            actionSnake: _.snakeCase(request.action),
            description: convertNewlines(request.description),
            parameters: allParameters,
            parametersList,
            returnType,
            returnResult,
        };

        const template = Handlebars.compile(methodTemplate);
        return template(methodData);
    }).join('\n');
};

const classTemplate = `
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

    {{{methods}}}
}
`;

// Generate methods from the JSON schema
const methods = generateMethods(data.requests);

// Generate the complete class code
const template = Handlebars.compile(classTemplate);
const classCode = template({ methods });

// Write the generated code to a Rust file
fs.writeFileSync(__dirname + '/src/lib.rs', classCode);

console.log('Rust library code generated successfully.');
