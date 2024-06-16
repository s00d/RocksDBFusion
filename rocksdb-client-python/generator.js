const fs = require('fs');
const Handlebars = require('handlebars');
const _ = require('lodash');

// Объект для конвертации типов
const typeConversion = {
    'usize': 'int',
    'u32': 'int',
    'bool': 'bool',
    'string': 'str',
};

// Функция для конвертации типов
const convertType = (type) => {
    return typeConversion[type] || type;
};

// Функция для преобразования переносов строк
const convertNewlines = (text) => {
    return text ? text.replace(/\\n/g, '\n     * ') : '';
};

// Рекурсивная функция для обработки параметров с properties
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
            });
        }
    }
    return result;
};

// Шаблон для генерации методов класса
const methodTemplate = `
    async def {{action_snake}}(self, {{{parameters_list}}}):
        """
        {{{description}}}
        {{#each parameters}}
        @param {{{param.param_type}}} {{replace name 'options.' ''}}: {{{description}}}
        {{/each}}
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "{{{action}}}",
            "options": {},
        }

        {{#each required_parameters}}
        if "options." in "{{{name}}}":
            request["options"]["{{{replace name 'options.' ''}}}"] = {{{replace name 'options.' ''}}}
        else:
            request["{{{name}}}"] = {{{replace name 'options.' ''}}}
        {{/each}}

        {{#each optional_parameters}}
        if {{{replace name 'options.' ''}}} is not None:
            if "options." in "{{{name}}}":
                request["options"]["{{{replace name 'options.' ''}}}"] = {{{replace name 'options.' ''}}}
            else:
                request["{{{name}}}"] = {{{replace name 'options.' ''}}}
        {{/each}}

        response = await self.send_request(request)
        return self.handle_response(response)
`;

// Регистрируем кастомный хелпер для проверки вхождения строки
Handlebars.registerHelper('contains', (haystack, needle) => haystack.includes(needle));

// Регистрируем кастомный хелпер для замены части строки
Handlebars.registerHelper('replace', (haystack, needle, replacement) => haystack.replace(needle, replacement));

// Чтение JSON-данных
const data = JSON.parse(fs.readFileSync(__dirname + '/../requests_schema.json', 'utf8'));

// Генерация методов на основе JSON
const generateMethods = (requests) => {
    return requests.map(request => {
        const allParameters = processParameters(request.parameters);

        const parametersList = allParameters.map(param => {
            const defaultValue = param.required ? '' : ' = None';
            const nullValue = param.required ? '' : '|None';
            return `${param.name.replace('options.', '')}: ${param.param_type}${nullValue}${defaultValue}`;
        }).join(', ');

        const methodData = {
            action: request.action,
            action_snake: _.snakeCase(request.action),
            description: convertNewlines(request.description),
            parameters: allParameters,
            required_parameters: allParameters.filter(param => param.required),
            optional_parameters: allParameters.filter(param => !param.required),
            parameters_list: parametersList,
        };

        const template = Handlebars.compile(methodTemplate, {noEscape: true});
        return template(methodData);
    }).join('\n');
};

// Основной шаблон класса
const classTemplate = `
import asyncio
import socket
import json

class RocksDBClient:
    def __init__(self, host: str, port: int, token: str = None, timeout: int = 20, retry_interval: int = 2):
        self.host = host
        self.port = port
        self.token = token
        self.timeout = timeout
        self.retry_interval = retry_interval
        self.socket = None

    async def connect(self):
        start_time = asyncio.get_event_loop().time()

        while True:
            try:
                self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                await asyncio.get_event_loop().sock_connect(self.socket, (self.host, self.port))
                return  # Connection successful
            except Exception as error:
                if (asyncio.get_event_loop().time() - start_time) >= self.timeout:
                    raise Exception(f"Unable to connect to server: {error}")
                await asyncio.sleep(self.retry_interval)

    def close(self):
        if self.socket:
            self.socket.close()
            self.socket = None

    async def send_request(self, request):
        if not self.socket:
            await self.connect()

        if self.token is not None:
            request['token'] = self.token  # Add token to request if present

        request_json = json.dumps(request) + "\\n"
        self.socket.sendall(request_json.encode('utf-8'))

        response_json = await self.read_socket()
        response = json.loads(response_json)

        if response is None:
            raise Exception("Invalid response from server")

        return response

    async def read_socket(self):
        data = b''
        while True:
            chunk = await asyncio.get_event_loop().sock_recv(self.socket, 4096)
            data += chunk
            if b"\\n" in data:
                break
        return data.decode('utf-8')

    def handle_response(self, response):
        if response['success'] and 'result' in response:
            return response['result']
        raise Exception(response['error'])

    {{ methods }}
`;

const methods = generateMethods(data.requests);

// Генерация полного кода класса
const template = Handlebars.compile(classTemplate, {noEscape: true});
const classCode = template({ methods });

// Запись в файл
fs.writeFileSync(__dirname + '/src/rocksdb_client.py', classCode);

console.log('Python code generated successfully.');
