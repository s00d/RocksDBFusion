import { createConnection } from "net";
import type {Socket} from "node:net";

interface RocksDBResponse {
    success: boolean;
    result?: string;
}

class RocksDBClient {
    host: string;
    port: number;
    token: string | null;
    socket: any;
    timeout: number;
    retryInterval: number;
    private pool: any[];
    private maxActiveConnections: number;
    private activeConnections: number;
    private waitingQueue: ((value: Socket) => void)[];

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
        this.pool = [];
        this.maxActiveConnections = 10;
        this.activeConnections = 0;
        this.waitingQueue = [];
    }

    /**
     * Closes all connections in the pool.
     */
    close(): void {
        for (const socket of this.pool) {
            socket.end();
        }
        this.pool = [];
    }

    private async createSocket(host: string, port: number): Promise<Socket> {
        return new Promise((resolve, reject) => {
            const socket = createConnection({ host, port }, () => {
                socket.setMaxListeners(3000);
                resolve(socket);
            });
            socket.on('error', reject);
        });
    }

    private async getConnection(): Promise<Socket> {
        if (this.pool.length > 0) {
            return this.pool.pop();
        }

        if (this.activeConnections < this.maxActiveConnections) {
            this.activeConnections++;
            return this.createSocket(this.host, this.port);
        }

        // If the maximum number of active connections is reached, wait for a connection to be released
        return new Promise((resolve) => {
            this.waitingQueue.push(resolve);
        });
    }

    private releaseConnection(socket: Socket) {
        if (this.waitingQueue.length > 0) {
            // If there are waiting requests, resolve the first one
            const resolve = this.waitingQueue.shift();
            if (resolve) {
                resolve(socket);
            }
        } else {
            this.pool.push(socket);
            this.activeConnections--;
        }
    }

    async sendRequest(request: object): Promise<RocksDBResponse> {
        if (this.token !== null) {
            (request as any).token = this.token;
        }

        const requestData = JSON.stringify(request); // Use JSON encoding
        const socket = await this.getConnection();
        socket.write(Buffer.concat([Buffer.from(requestData), Buffer.from('\n')]));

        return new Promise((resolve, reject) => {
            let dataBuffer: Buffer[] = [];

            const onData = (data: Buffer) => {
                dataBuffer.push(data);

                const responseBuffer = Buffer.concat(dataBuffer);
                const separatorIndex = responseBuffer.indexOf('\n');

                if (separatorIndex !== -1) {
                    const completeMessage = responseBuffer.slice(0, separatorIndex).toString();
                    const remainingBuffer = responseBuffer.slice(separatorIndex + 1);

                    try {
                        const response: RocksDBResponse = JSON.parse(completeMessage); // Use JSON decoding
                        socket.removeListener('data', onData);
                        this.releaseConnection(socket);
                        resolve(response);
                    } catch (error) {
                        console.error("Failed to decode response: {}", error);
                        reject(new Error("Failed to decode response"));
                    }

                    dataBuffer = [];
                    if (remainingBuffer.length > 0) {
                        dataBuffer.push(remainingBuffer);
                    }
                }
            };

            socket.on('data', onData);

            socket.once('error', (err: Error) => {
                socket.removeListener('data', onData);
                this.releaseConnection(socket);
                reject(err);
            });

            socket.once('close', () => {
                socket.removeListener('data', onData);
                this.releaseConnection(socket);
                reject(new Error("Connection closed before response was received"));
            });
        });
    }

    /**
     * Handles the response from the server.
     *
     * @param {object} response The response from the server.
     * @return {any} The result from the response.
     * @throws {Error} If the response indicates an error.
     */
    handleResponse(response: RocksDBResponse): string|null|undefined {
        if (response.success) {
            return response.result;
        }

        throw new Error(response.result);
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
    async put(key: string , value: string , cf_name: string|null  = null, txn: boolean|null  = null) {
      const request: any = {
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
    async get(key: string , cf_name: string|null  = null, default_value: string|null  = null, txn: boolean|null  = null) {
      const request: any = {
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
    async delete(key: string , cf_name: string|null  = null, txn: boolean|null  = null) {
      const request: any = {
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
    async merge(key: string , value: string , cf_name: string|null  = null, txn: boolean|null  = null) {
      const request: any = {
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
    async getProperty(value: string , cf_name: string|null  = null) {
      const request: any = {
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
    async keys(start: string , limit: string , query: string|null  = null) {
      const request: any = {
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
    async all(query: string|null  = null) {
      const request: any = {
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
      const request: any = {
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
    async createColumnFamily(cf_name: string ) {
      const request: any = {
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
    async dropColumnFamily(cf_name: string ) {
      const request: any = {
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
    async compactRange(start: string|null  = null, end: string|null  = null, cf_name: string|null  = null) {
      const request: any = {
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
    async writeBatchPut(key: string , value: string , cf_name: string|null  = null) {
      const request: any = {
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
    async writeBatchMerge(key: string , value: string , cf_name: string|null  = null) {
      const request: any = {
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
    async writeBatchDelete(key: string , cf_name: string|null  = null) {
      const request: any = {
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
      const request: any = {
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
      const request: any = {
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
      const request: any = {
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
      const request: any = {
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
    async destroyIterator(iterator_id: string ) {
      const request: any = {
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
    async iteratorSeek(iterator_id: string , key: string ) {
      const request: any = {
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
    async iteratorNext(iterator_id: string ) {
      const request: any = {
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
    async iteratorPrev(iterator_id: string ) {
      const request: any = {
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
      const request: any = {
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
      const request: any = {
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
    async restore(backup_id: string ) {
      const request: any = {
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
      const request: any = {
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
      const request: any = {
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
      const request: any = {
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
      const request: any = {
          action: 'rollback_transaction',
          options: {},
      };



      const response = await this.sendRequest(request);
      return this.handleResponse(response);
    }

}

// Export the class
export default RocksDBClient;