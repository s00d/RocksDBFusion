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
    return text ? text.replace(/\\n/g, '\n        ') : '';
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
                param_type_normal: convertType(param.param_type.toLowerCase()),
                required: param.required,
                notRequired: !param.required,
                inOptions: parentKey === 'options'
            });
        }
    }
    return result;
};

// Чтение шаблонов
const methodTemplate = fs.readFileSync(__dirname + '/templates/methodTemplate.hbs', 'utf8');
const classTemplate = fs.readFileSync(__dirname + '/templates/classTemplate.hbs', 'utf8');

Handlebars.registerHelper('replace', (haystack, needle, replacement) => haystack.replace(needle, replacement));
Handlebars.registerHelper('snake_case', (str) => _.snakeCase(str));
Handlebars.registerHelper('contains', (haystack, needle) => haystack.includes(needle));
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
            actionSnake: _.snakeCase(request.action),
            actionCamel: _.camelCase(request.action),
            description: convertNewlines(request.description),
            parameters: allParameters,
            requiredParameters: allParameters.filter(param => param.required),
            optionalParameters: allParameters.filter(param => !param.required),
            parametersList,
        };

        const template = Handlebars.compile(methodTemplate, { noEscape: true });
        return template(methodData);
    }).join('\n');
};

// Генерация методов
const methods = generateMethods(data.requests);

// Генерация полного кода класса
const template = Handlebars.compile(classTemplate, { noEscape: true });
const classCode = template({ methods });

// Запись в файл
fs.writeFileSync(__dirname + '/src/rocksdb_client.py', classCode);

console.log('Python code generated successfully.');
