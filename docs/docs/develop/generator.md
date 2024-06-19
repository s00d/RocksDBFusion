---
lang: en-US
icon: fas fa-cogs
category:
  - DEVELOP
---

# Code Generator

## Overview

This documentation explains the logic and usage of the code generator for creating clients in various programming languages from comments in Rust source code. The generator reads comments from the Rust source file, converts them into a JSON schema, and then uses Handlebars templates to generate client code in different languages.

## How It Works

### Step 1: Extracting Schema from Rust Source Code

The generator reads the Rust source file and extracts the schema information from specially formatted comments. This information includes the action, description, parameters, and responses for each request.

### Step 2: Generating JSON Schema

The extracted schema is then converted into a JSON format and saved as `requests_schema.json` in the root of the project.

### Step 3: Generating Client Code

Using Handlebars templates, the generator reads the JSON schema and generates client code for the specified language (e.g., Node.js, Python, Rust).

## Setup

To run the generators, you need to have Node.js installed. The project structure should include a `package.json` file with the necessary scripts and dependencies.

### Installation

Clone the repository and install the dependencies:

```bash
git clone <repository_url>
cd <repository_directory>
npm install
```

## Scripts

### Generate Schema

To generate the JSON schema from the Rust source code, run:

```bash
npm run schema
```

### Generate Client Code

To generate client code for a specific language, run one of the following commands:

```bash
npm run client:node  # Generates Node.js client code
npm run client:python  # Generates Python client code
npm run client:rust  # Generates Rust client code
```

To generate client code for all supported languages, run:

```bash
npm run client:all
```

## Adding a New Generator

### Step 1: Create a New Directory

Create a new directory for your language generator in the project root. For example, if you want to add a Go generator, create a directory named `rocksdb-client-go`.

### Step 2: Create Handlebars Templates

Inside your new directory, create the necessary Handlebars templates for generating the client code. For example:

```plaintext
rocksdb-client-go/
  ├── generator.js
  ├── templates/
  │   ├── methodTemplate.hbs
  │   └── classTemplate.hbs
```

### Step 3: Write the Generator Script

Create a `generator.js` file inside your new directory. This script will read the JSON schema and generate the client code using the Handlebars templates.

Example `generator.js`:

```javascript
const fs = require('fs');
const Handlebars = require('handlebars');
const _ = require('lodash');

// Function to convert types
const typeConversion = {
  usize: 'int',
  u32: 'int',
  bool: 'bool',
  string: 'string',
};

const convertType = (type) => typeConversion[type] || type;

const convertNewlines = (text) => (text ? text.replace(/\\n/g, '\n     * ') : '');

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

// Reading templates
const methodTemplate = fs.readFileSync(__dirname + '/templates/methodTemplate.hbs', 'utf8');
const classTemplate = fs.readFileSync(__dirname + '/templates/classTemplate.hbs', 'utf8');

Handlebars.registerHelper('replace', (haystack, needle, replacement) => haystack.replace(needle, replacement));
Handlebars.registerHelper('snake_case', (str) => _.snakeCase(str));
Handlebars.registerHelper('contains', (haystack, needle) => haystack.includes(needle));

// Reading JSON data
const data = JSON.parse(fs.readFileSync(__dirname + '/../requests_schema.json', 'utf8'));

// Generating methods based on JSON
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

// Generating methods
const methods = generateMethods(data.requests);

// Generating full class code
const template = Handlebars.compile(classTemplate, { noEscape: true });
const classCode = template({ methods });

// Writing to file
fs.writeFileSync(__dirname + '/src/index.ts', classCode);

console.log('Go code generated successfully.');
```

### Step 4: Update `package.json`

Add a new script in the `package.json` file to run your new generator:

```json
"scripts": {
  "client:go": "node rocksdb-client-go/generator.js",
  "client:all": "npm run client:node && npm run client:python && npm run client:rust && npm run client:go"
}
```

### Step 5: Run Your New Generator

Run the generator for your new language to generate the client code:

```bash
npm run client:go
```

## JSON Schema

The JSON schema is generated from the Rust source code comments and provides a structured representation of the API, including actions, parameters, and responses.

### Schema Example

Here is an example of the JSON schema generated from the Rust comments:

```json
{
  "requests": [
    {
      "action": "iterator_seek",
      "description": "Seeks to a specific key in the iterator.\\n\\nThis function handles the `iterator_seek` action which seeks to a specified key in an existing iterator in the RocksDB database.\\nThe function requires the ID of the iterator, the key to seek, and the direction of the seek (Forward or Reverse).",
      "parameters": {
        "options.iterator_id": {
          "param_type": "String",
          "required": true,
          "description": "The iterator ID"
        },
        "key": {
          "param_type": "String",
          "required": true,
          "description": "The key to seek"
        }
      },
      "response": {
        "success": {
          "param_type": "bool",
          "required": true,
          "description": "Whether the operation was successful"
        },
        "result": {
          "param_type": "Option<String>",
          "required": false,
          "description": "The result of the operation"
        },
        "error": {
          "param_type": "Option<String>",
          "required": false,
          "description": "Any error that occurred"
        }
      }
    }
  ]
}
```

### Schema Description

| Field         | Type     | Description                                     |
|---------------|----------|-------------------------------------------------|
| `action`      | `string` | The action name for the request.                |
| `description` | `string` | A detailed description of the action.           |
| `parameters`  | `object` | An object containing parameters for the action. |
| `response`    | `object` | An object containing the response structure.    |

### Generating the JSON Schema

To generate the JSON schema, run the following command:

```bash
npm run schema
```
### Example Rust Comment for JSON Schema

Here is an example Rust comment that will be parsed to generate the JSON schema:

```rust
/**
 * Seeks to a specific key in the iterator.
 *
 * This function handles the `iterator_seek` action which seeks to a specified key in an existing iterator in the RocksDB database.
 * The function requires the ID of the iterator, the key to seek, and the direction of the seek (Forward or Reverse).
 *
 * # Link: iterator_seek
 *
 * # Parameters
 * - `options.iterator_id`: String - The iterator ID
 * - `key`: String - The key to seek
 *
 * # Returns
 * - `success`: bool - Whether the operation was successful
 * - `result`: Option<String> - The result of the operation
 * - `error`: Option<String> - Any error that occurred
 */
async fn handle_iterator_seek(
    &self,
    req: Request,
    direction: rust_rocksdb::Direction,
) -> Response {
    debug!(
        "handle_iterator_seek with iterator_id: {:?}, key: {:?}",
        req.parse_option::<usize>("iterator_id"),
        req.key
    );
    let iterator_id = req.parse_option::<usize>("iterator_id").unwrap_or(0);
    match req.key {
        Some(key) => self
            .db_manager
            .iterator_seek(iterator_id, key, direction)
            .map(|result| Response {
                success: true,
                result: Some(result),
                error: None,
            })
            .unwrap_or_else(|e| Response {
                success: false,
                result: None,
                error: Some(e),
            }),
        None => Response {
            success: false,
            result: None,
            error: Some("Missing key".to_string()),
        },
    }
}
```

### Comment Description

The Rust comments used to generate the JSON schema must follow a specific format. Here is a breakdown of the required components:

1. **Function Description**: A brief description of what the function does.

   ```rust
   /**
    * Seeks to a specific key in the iterator.
    */
   ```

2. **Detailed Description**: A more detailed explanation of the function's behavior.

   ```rust
   /**
    * This function handles the `iterator_seek` action which seeks to a specified key in an existing iterator in the RocksDB database.
    * The function requires the ID of the iterator, the key to seek, and the direction of the seek (Forward or Reverse).
    */
   ```

3. **Link**: The action name that maps to this function. This is used to identify the function in the schema.

   ```rust
   /**
    * # Link: iterator_seek
    */
   ```

4. **Parameters**: A list of parameters required by the function. Each parameter is documented with its type and a brief description. If the parameter is part of the `options` object, it should be prefixed with `options.`.

   ```rust
   /**
    * # Parameters
    * - `options.iterator_id`: String - The iterator ID
    * - `key`: String - The key to seek
    */
   ```

5. **Returns**: The structure of the response object, detailing each field, its type, and its description.

   ```rust
   /**
    * # Returns
    * - `success`: bool - Whether the operation was successful
    * - `result`: Option<String> - The result of the operation
    * - `error`: Option<String> - Any error that occurred
    */
   ```

By adhering to this comment format, the generator can accurately extract the necessary information to create a JSON schema, which is then used to generate client code for different languages.

The above comment in the Rust code will be parsed to generate the corresponding JSON schema, which will then be used to generate client code in different languages.

By following these steps and examples, you can extend the code generator to support any programming language and maintain consistency in client code generation across different platforms.