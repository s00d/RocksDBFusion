const fs = require('fs');
const Handlebars = require('handlebars');
const _ = require('lodash');

// Объект для конвертации типов
const typeConversion = {
  usize: 'number',
  u32: 'number',
  bool: 'boolean',
  string: 'string',
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
        param_type: "{" + convertType(param.param_type.toLowerCase()) + '}',
        param_type_normal: convertType(param.param_type.toLowerCase()),
        description: param.description,
        required: param.required,
      });
    }
  }
  return result;
};
// Шаблон для генерации методов класса
const methodTemplate = `
    /**
     * {{{description}}}
     *
     * {{#each parameters}}@param {{param_type}} {{replace name 'options.' ''}} {{description}}
     * {{/each}}
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async {{actionCamel}}({{{parametersList}}}) {
      const request: any = {
          action: '{{action}}',
          options: {},
      };

      {{#each requiredParameters}}
      {{#if (contains name "options.")}}
      request.options['{{replace name 'options.' ''}}'] = {{replace name 'options.' ''}};
      {{else}}
      request['{{name}}'] = {{replace name 'options.' ''}};
      {{/if}}
      {{/each}}

      {{#each optionalParameters}}
      if ({{replace name 'options.' ''}} !== null) {
          {{#if (contains name "options.")}}
          request.options['{{replace name 'options.' ''}}'] = {{replace name 'options.' ''}};
          {{else}}
          request['{{name}}'] = {{replace name 'options.' ''}};
          {{/if}}
      }
      {{/each}}

      const response = await this.sendRequest(request);
      return this.handleResponse(response);
    }
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
      const defaultValue = param.required ? '' : ' = null';
      const nullValue = param.required ? '' : '|null';
      return `${param.name.replace('options.', '')}: ${param.param_type_normal}${nullValue} ${defaultValue}`;
    }).join(', ');

    const methodData = {
      action: request.action,
      actionCamel: _.camelCase(request.action),
      description: convertNewlines(request.description),
      parameters: allParameters,
      requiredParameters: allParameters.filter(param => param.required),
      optionalParameters: allParameters.filter(param => !param.required),
      parametersList,
    };

    const template = Handlebars.compile(methodTemplate);
    return template(methodData);
  }).join('\n');
};

// Основной шаблон класса
const classTemplate = `interface RocksDBResponse {
    success: boolean;
    result?: string;
    error?: string;
}

class RocksDBClient {
    host: string;
    port: number;
    token: string | null;
    socket: any;
    timeout: number;
    retryInterval: number;

    /**
     * Constructor to initialize the RocksDB client.
     *
     * @param {string} host The host of the RocksDB server.
     * @param {number} port The port of the RocksDB server.
     * @param {string|null} [token] Optional authentication token for the RocksDB server.
     * @param {number} [timeout=20] Timeout in seconds.
     * @param {number} [retryInterval=2] Retry interval in seconds.
     */
    constructor(host: string, port: number, token: string | null = null, timeout: number = 20, retryInterval: number = 2) {
        this.host = host;
        this.port = port;
        this.token = token;
        this.timeout = timeout;
        this.retryInterval = retryInterval;
        this.socket = null;
    }

    /**
     * Connects to the RocksDB server with retry mechanism.
     *
     * @throws {Error} If unable to connect to the server.
     */
    async connect(): Promise<void> {
        const startTime = Date.now();

        while (true) {
            try {
                this.socket = await this.createSocket(this.host, this.port);
                return; // Connection successful
            } catch (error: any) {
                if ((Date.now() - startTime) >= this.timeout * 1000) {
                    throw new Error(\`Unable to connect to server: \${error.message}\`);
                }
                await this.sleep(this.retryInterval * 1000);
            }
        }
    }
    
    /**
     * Closes the socket connection.
     */
    close(): void {
        if (this.socket) {
            this.socket.end();
            this.socket = null;
        }
    }

    /**
     * Creates a socket connection.
     * @private
     */
    createSocket(host: string, port: number): Promise<any> {
        return new Promise((resolve, reject) => {
            const socket = require('net').createConnection({ host, port }, () => {
                resolve(socket);
            });
            socket.on('error', reject);
        });
    }

    /**
     * Sleeps for the given number of milliseconds.
     * @private
     */
    sleep(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    /**
     * Sends a request to the RocksDB server.
     *
     * @param {object} request The request to be sent.
     * @return {Promise<object>} The response from the server.
     * @throws {Error} If the response from the server is invalid.
     */
    async sendRequest(request: object): Promise<RocksDBResponse> {
        if (!this.socket) {
            await this.connect();
        }

        if (this.token !== null) {
            (request as any).token = this.token; // Add token to request if present
        }

        const requestJson = JSON.stringify(request) + "\\n";
        this.socket.write(requestJson);

        const responseJson = await this.readSocket();
        const response = JSON.parse(responseJson);

        if (response === null) {
            throw new Error("Invalid response from server");
        }

        return response;
    }

    /**
     * Reads data from the socket.
     * @private
     */
    readSocket(): Promise<string> {
        return new Promise((resolve, reject) => {
            let data = '';
            this.socket.on('data', (chunk: string) => {
                data += chunk;
                if (data.includes("\\n")) {
                    resolve(data);
                }
            });
            this.socket.on('error', reject);
        });
    }

    /**
     * Handles the response from the server.
     *
     * @param {object} response The response from the server.
     * @return {any} The result from the response.
     * @throws {Error} If the response indicates an error.
     */
    handleResponse(response: RocksDBResponse): string|null {
        if (response.success && response.result !== undefined) {
            return response.result;
        }

        throw new Error(response.error);
    }

    {{{methods}}}
}

// Export the class
export default RocksDBClient;
`;

// Генерация методов
    const methods = generateMethods(data.requests);

// Генерация полного кода класса
    const template = Handlebars.compile(classTemplate);
    const classCode = template({ methods });

// Запись в файл
    fs.writeFileSync(__dirname + '/src/index.ts', classCode);

    console.log('TypeScript code generated successfully.');
