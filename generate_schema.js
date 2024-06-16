const fs = require('fs');
const path = require('path');
const _ = require('lodash');

class Parameter {
    constructor(param_type, required, description, properties = null) {
        this.param_type = param_type;
        this.required = required;
        this.description = description;
        if (properties) {
            this.properties = properties;
        }
    }
}

class RequestSchema {
    constructor(action, description, parameters, response) {
        this.action = action;
        this.description = description;
        this.parameters = parameters;
        this.response = response;
    }
}

class Schema {
    constructor(requests) {
        this.requests = requests;
    }
}

function insertNestedParam(map, key, param) {
    const parts = key.split('.');
    if (parts.length === 1) {
        map[key] = param;
    } else {
        const nestedKey = parts[0];
        const nestedParamKey = parts.slice(1).join('.');

        if (!map[nestedKey]) {
            map[nestedKey] = new Parameter('object', false, '', {});
        }

        const nestedParam = map[nestedKey];
        if (nestedParam.properties) {
            insertNestedParam(nestedParam.properties, nestedParamKey, param);
        }
    }
}

function main() {
    const sourceFile = path.join(__dirname, './server/src/server.rs');
    const content = fs.readFileSync(sourceFile, 'utf8');

    const startHandleRe = /\/\*\*/;
    const endHandleRe = /\*\//;
    const linkRe = /\* # Link: (\w+)/;
    const paramRe = /\* - `(\w+(?:\.\w+)?)`: (\w+(?:<\w+>)?) - (.+)/;
    const returnsRe = /\* # Returns/;
    const descriptionLineRe = /\* (.+)/;
    const parametersRe = /\* # Parameters/;

    let insideHandle = false;
    let currentAction = '';
    let currentDescription = '';
    let currentParams = {};
    let currentResponse = {};
    let parsingReturns = false;
    let descriptionLines = [];
    let parsingDescription = false;

    const requests = [];

    content.split('\n').forEach(line => {
        if (startHandleRe.test(line)) {
            insideHandle = true;
            currentParams = {};
            currentResponse = {};
            parsingReturns = false;
            descriptionLines = [];
            parsingDescription = true;
            return;
        }

        if (endHandleRe.test(line)) {
            insideHandle = false;
            if (descriptionLines.length > 0) {
                currentDescription = descriptionLines.join('\\n');
            }
            requests.push(new RequestSchema(
                currentAction,
                currentDescription,
                currentParams,
                currentResponse
            ));
            return;
        }

        if (insideHandle) {
            if (linkRe.test(line)) {
                currentAction = line.match(linkRe)[1];
                parsingDescription = false;
            }

            if (parsingDescription) {
                const match = line.match(descriptionLineRe);
                if (match) {
                    descriptionLines.push(match[1]);
                }
            }

            if (parametersRe.test(line)) {
                parsingDescription = false;
                return;
            }

            if (returnsRe.test(line)) {
                parsingReturns = true;
                return;
            }

            if (paramRe.test(line)) {
                const match = line.match(paramRe);
                const paramName = match[1];
                const paramTypeFull = match[2];
                const paramDescription = match[3];
                const required = !paramTypeFull.startsWith('Option');

                const paramType = paramTypeFull.startsWith('Option<') ?
                    paramTypeFull.slice(7, -1) : paramTypeFull;

                const parameter = new Parameter(paramType, required, paramDescription);

                if (parsingReturns) {
                    insertNestedParam(currentResponse, paramName, parameter);
                } else {
                    insertNestedParam(currentParams, paramName, parameter);
                }
            }
        }
    });

    const schema = new Schema(requests);
    const json = JSON.stringify(schema, null, 2);
    fs.writeFileSync(path.join(__dirname, './requests_schema.json'), json);

    console.log('JSON schema generated successfully.');
}

main();
