"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class RocksDBClient {
    /**
     * Constructor to initialize the RocksDB client.
     *
     * @param {string} host The host of the RocksDB server.
     * @param {number} port The port of the RocksDB server.
     * @param {string|null} [token] Optional authentication token for the RocksDB server.
     * @param {number} [timeout=20] Timeout in seconds.
     * @param {number} [retryInterval=2] Retry interval in seconds.
     */
    constructor(host, port, token = null, timeout = 20, retryInterval = 2) {
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
    async connect() {
        const startTime = Date.now();
        while (true) {
            try {
                this.socket = await this.createSocket(this.host, this.port);
                return; // Connection successful
            }
            catch (error) {
                if ((Date.now() - startTime) >= this.timeout * 1000) {
                    throw new Error(`Unable to connect to server: ${error.message}`);
                }
                await this.sleep(this.retryInterval * 1000);
            }
        }
    }
    /**
     * Closes the socket connection.
     */
    close() {
        if (this.socket) {
            this.socket.end();
            this.socket = null;
        }
    }
    /**
     * Creates a socket connection.
     * @private
     */
    createSocket(host, port) {
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
    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
    /**
     * Sends a request to the RocksDB server.
     *
     * @param {object} request The request to be sent.
     * @return {Promise<object>} The response from the server.
     * @throws {Error} If the response from the server is invalid.
     */
    async sendRequest(request) {
        if (!this.socket) {
            await this.connect();
        }
        if (this.token !== null) {
            request.token = this.token; // Add token to request if present
        }
        const requestJson = JSON.stringify(request) + "\n";
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
    readSocket() {
        return new Promise((resolve, reject) => {
            let data = '';
            this.socket.on('data', (chunk) => {
                data += chunk;
                if (data.includes("\n")) {
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
    handleResponse(response) {
        if (response.success && response.result !== undefined) {
            return response.result;
        }
        throw new Error(response.error);
    }
    /**
     * Inserts a key-value pair into the database.
     * This function handles the `put` action which inserts a specified key-value pair into the RocksDB database.
     * The function can optionally operate within a specified column family and transaction if provided.
     *
     * @param {string} key The key to put
     * @param {string} value The value to put
     * @param {string} cf_name The column family name
     * @param {boolean} txn The transaction ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async put(key, value, cf_name = null, txn = null) {
        const request = {
            action: 'put',
            options: {},
        };
        request['key'] = key;
        request['value'] = value;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        if (txn !== null) {
            request['txn'] = txn;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Retrieves the value associated with a key from the database.
     * This function handles the `get` action which fetches the value associated with a specified key from the RocksDB database.
     * The function can optionally operate within a specified column family and return a default value if the key is not found.
     *
     * @param {string} key The key to get
     * @param {string} cf_name The column family name
     * @param {string} default_value The default value
     * @param {boolean} txn The transaction ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async get(key, cf_name = null, default_value = null, txn = null) {
        const request = {
            action: 'get',
            options: {},
        };
        request['key'] = key;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        if (default_value !== null) {
            request['default_value'] = default_value;
        }
        if (txn !== null) {
            request['txn'] = txn;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Deletes a key-value pair from the database.
     * This function handles the `delete` action which removes a specified key-value pair from the RocksDB database.
     * The function can optionally operate within a specified column family and transaction if provided.
     *
     * @param {string} key The key to delete
     * @param {string} cf_name The column family name
     * @param {boolean} txn The transaction ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async delete(key, cf_name = null, txn = null) {
        const request = {
            action: 'delete',
            options: {},
        };
        request['key'] = key;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        if (txn !== null) {
            request['txn'] = txn;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Merges a value with an existing key in the database.
     * This function handles the `merge` action which merges a specified value with an existing key in the RocksDB database.
     * The function can optionally operate within a specified column family and transaction if provided.
     *
     * @param {string} key The key to merge
     * @param {string} value The value to merge
     * @param {string} cf_name The column family name
     * @param {boolean} txn The transaction ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async merge(key, value, cf_name = null, txn = null) {
        const request = {
            action: 'merge',
            options: {},
        };
        request['key'] = key;
        request['value'] = value;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        if (txn !== null) {
            request['txn'] = txn;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Retrieves a property of the database.
     * This function handles the `get_property` action which fetches a specified property of the RocksDB database.
     * The function can optionally operate within a specified column family if provided.
     *
     * @param {string} value The property to get
     * @param {string} cf_name The column family name
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async getProperty(value, cf_name = null) {
        const request = {
            action: 'get_property',
            options: {},
        };
        request['value'] = value;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Retrieves a range of keys from the database.
     * This function handles the `keys` action which retrieves a range of keys from the RocksDB database.
     * The function can specify a starting index, limit on the number of keys, and a query string to filter keys.
     *
     * @param {string} start The start index
     * @param {string} limit The limit of keys to retrieve
     * @param {string} query The query string to filter keys
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async keys(start, limit, query = null) {
        const request = {
            action: 'keys',
            options: {},
        };
        request.options['start'] = start;
        request.options['limit'] = limit;
        if (query !== null) {
            request.options['query'] = query;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Retrieves all keys from the database.
     * This function handles the `all` action which retrieves all keys from the RocksDB database.
     * The function can specify a query string to filter keys.
     *
     * @param {string} query The query string to filter keys
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async all(query = null) {
        const request = {
            action: 'all',
            options: {},
        };
        if (query !== null) {
            request.options['query'] = query;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Lists all column families in the database.
     * This function handles the `list_column_families` action which lists all column families in the RocksDB database.
     * The function requires the path to the database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async listColumnFamilies() {
        const request = {
            action: 'list_column_families',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Creates a new column family in the database.
     * This function handles the `create_column_family` action which creates a new column family in the RocksDB database.
     * The function requires the name of the column family to create.
     *
     * @param {string} cf_name The column family name to create
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async createColumnFamily(cf_name) {
        const request = {
            action: 'create_column_family',
            options: {},
        };
        request['cf_name'] = cf_name;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Drops an existing column family from the database.
     * This function handles the `drop_column_family` action which drops an existing column family from the RocksDB database.
     * The function requires the name of the column family to drop.
     *
     * @param {string} cf_name The column family name to drop
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async dropColumnFamily(cf_name) {
        const request = {
            action: 'drop_column_family',
            options: {},
        };
        request['cf_name'] = cf_name;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Compacts a range of keys in the database.
     * This function handles the `compact_range` action which compacts a specified range of keys in the RocksDB database.
     * The function can optionally specify the start key, end key, and column family.
     *
     * @param {string} start The start key
     * @param {string} end The end key
     * @param {string} cf_name The column family name
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async compactRange(start = null, end = null, cf_name = null) {
        const request = {
            action: 'compact_range',
            options: {},
        };
        if (start !== null) {
            request.options['start'] = start;
        }
        if (end !== null) {
            request.options['end'] = end;
        }
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Adds a key-value pair to the current write batch.
     * This function handles the `write_batch_put` action which adds a specified key-value pair to the current write batch.
     * The function can optionally operate within a specified column family.
     *
     * @param {string} key The key to put
     * @param {string} value The value to put
     * @param {string} cf_name The column family name
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async writeBatchPut(key, value, cf_name = null) {
        const request = {
            action: 'write_batch_put',
            options: {},
        };
        request['key'] = key;
        request['value'] = value;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Merges a value with an existing key in the current write batch.
     * This function handles the `write_batch_merge` action which merges a specified value with an existing key in the current write batch.
     * The function can optionally operate within a specified column family.
     *
     * @param {string} key The key to merge
     * @param {string} value The value to merge
     * @param {string} cf_name The column family name
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async writeBatchMerge(key, value, cf_name = null) {
        const request = {
            action: 'write_batch_merge',
            options: {},
        };
        request['key'] = key;
        request['value'] = value;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Deletes a key from the current write batch.
     * This function handles the `write_batch_delete` action which deletes a specified key from the current write batch.
     * The function can optionally operate within a specified column family.
     *
     * @param {string} key The key to delete
     * @param {string} cf_name The column family name
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async writeBatchDelete(key, cf_name = null) {
        const request = {
            action: 'write_batch_delete',
            options: {},
        };
        request['key'] = key;
        if (cf_name !== null) {
            request['cf_name'] = cf_name;
        }
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Writes the current write batch to the database.
     * This function handles the `write_batch_write` action which writes the current write batch to the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async writeBatchWrite() {
        const request = {
            action: 'write_batch_write',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Clears the current write batch.
     * This function handles the `write_batch_clear` action which clears the current write batch.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async writeBatchClear() {
        const request = {
            action: 'write_batch_clear',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Destroys the current write batch.
     * This function handles the `write_batch_destroy` action which destroys the current write batch.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async writeBatchDestroy() {
        const request = {
            action: 'write_batch_destroy',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Creates a new iterator for the database.
     * This function handles the `create_iterator` action which creates a new iterator for iterating over the keys in the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async createIterator() {
        const request = {
            action: 'create_iterator',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Destroys an existing iterator.
     * This function handles the `destroy_iterator` action which destroys an existing iterator in the RocksDB database.
     * The function requires the ID of the iterator to destroy.
     *
     * @param {string} iterator_id The iterator ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async destroyIterator(iterator_id) {
        const request = {
            action: 'destroy_iterator',
            options: {},
        };
        request.options['iterator_id'] = iterator_id;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Seeks to a specific key in the iterator.
     * This function handles the `iterator_seek` action which seeks to a specified key in an existing iterator in the RocksDB database.
     * The function requires the ID of the iterator, the key to seek, and the direction of the seek (Forward or Reverse).
     *
     * @param {string} iterator_id The iterator ID
     * @param {string} key The key to seek
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async iteratorSeek(iterator_id, key) {
        const request = {
            action: 'iterator_seek',
            options: {},
        };
        request.options['iterator_id'] = iterator_id;
        request['key'] = key;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Advances the iterator to the next key.
     * This function handles the `iterator_next` action which advances an existing iterator to the next key in the RocksDB database.
     * The function requires the ID of the iterator.
     *
     * @param {string} iterator_id The iterator ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async iteratorNext(iterator_id) {
        const request = {
            action: 'iterator_next',
            options: {},
        };
        request.options['iterator_id'] = iterator_id;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Moves the iterator to the previous key.
     * This function handles the `iterator_prev` action which moves an existing iterator to the previous key in the RocksDB database.
     * The function requires the ID of the iterator.
     *
     * @param {string} iterator_id The iterator ID
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async iteratorPrev(iterator_id) {
        const request = {
            action: 'iterator_prev',
            options: {},
        };
        request.options['iterator_id'] = iterator_id;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Creates a backup of the database.
     * This function handles the `backup` action which creates a backup of the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async backup() {
        const request = {
            action: 'backup',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Restores the database from the latest backup.
     * This function handles the `restore_latest` action which restores the RocksDB database from the latest backup.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async restoreLatest() {
        const request = {
            action: 'restore_latest',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Restores the database from a specified backup.
     * This function handles the `restore` action which restores the RocksDB database from a specified backup.
     * The function requires the ID of the backup to restore.
     *
     * @param {string} backup_id The ID of the backup to restore
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async restore(backup_id) {
        const request = {
            action: 'restore',
            options: {},
        };
        request.options['backup_id'] = backup_id;
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Retrieves information about all backups.
     * This function handles the `get_backup_info` action which retrieves information about all backups of the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async getBackupInfo() {
        const request = {
            action: 'get_backup_info',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Begins a new transaction.
     * This function handles the `begin_transaction` action which begins a new transaction in the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async beginTransaction() {
        const request = {
            action: 'begin_transaction',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Commits an existing transaction.
     * This function handles the `commit_transaction` action which commits an existing transaction in the RocksDB database.
     * The function requires the ID of the transaction to commit.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async commitTransaction() {
        const request = {
            action: 'commit_transaction',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
    /**
     * Rolls back an existing transaction.
     * This function handles the `rollback_transaction` action which rolls back an existing transaction in the RocksDB database.
     * The function requires the ID of the transaction to roll back.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    async rollbackTransaction() {
        const request = {
            action: 'rollback_transaction',
            options: {},
        };
        const response = await this.sendRequest(request);
        return this.handleResponse(response);
    }
}
// Export the class
exports.default = RocksDBClient;
