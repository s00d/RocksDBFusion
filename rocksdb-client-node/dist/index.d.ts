interface RocksDBResponse {
    success: boolean;
    result?: string;
}
declare class RocksDBClient {
    host: string;
    port: number;
    token: string | null;
    socket: any;
    timeout: number;
    retryInterval: number;
    private pool;
    private maxActiveConnections;
    private activeConnections;
    private waitingQueue;
    /**
     * Constructor to initialize the RocksDB client.
     *
     * @param {string} host The host of the RocksDB server.
     * @param {number} port The port of the RocksDB server.
     * @param {string|null} [token] Optional authentication token for the RocksDB server.
     * @param {number} [timeout=20] Timeout in seconds.
     * @param {number} [retryInterval=2] Retry interval in seconds.
     */
    constructor(host: string, port: number, token?: string | null, timeout?: number, retryInterval?: number);
    /**
     * Closes all connections in the pool.
     */
    close(): void;
    private createSocket;
    private getConnection;
    private releaseConnection;
    sendRequest(request: object): Promise<RocksDBResponse>;
    /**
     * Handles the response from the server.
     *
     * @param {object} response The response from the server.
     * @return {any} The result from the response.
     * @throws {Error} If the response indicates an error.
     */
    handleResponse(response: RocksDBResponse): string | null | undefined;
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
    put(key: string, value: string, cf_name?: string | null, txn?: boolean | null): Promise<string | null | undefined>;
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
    get(key: string, cf_name?: string | null, default_value?: string | null, txn?: boolean | null): Promise<string | null | undefined>;
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
    delete(key: string, cf_name?: string | null, txn?: boolean | null): Promise<string | null | undefined>;
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
    merge(key: string, value: string, cf_name?: string | null, txn?: boolean | null): Promise<string | null | undefined>;
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
    getProperty(value: string, cf_name?: string | null): Promise<string | null | undefined>;
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
    keys(start: string, limit: string, query?: string | null): Promise<string | null | undefined>;
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
    all(query?: string | null): Promise<string | null | undefined>;
    /**
     * Lists all column families in the database.
     * This function handles the `list_column_families` action which lists all column families in the RocksDB database.
     * The function requires the path to the database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    listColumnFamilies(): Promise<string | null | undefined>;
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
    createColumnFamily(cf_name: string): Promise<string | null | undefined>;
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
    dropColumnFamily(cf_name: string): Promise<string | null | undefined>;
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
    compactRange(start?: string | null, end?: string | null, cf_name?: string | null): Promise<string | null | undefined>;
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
    writeBatchPut(key: string, value: string, cf_name?: string | null): Promise<string | null | undefined>;
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
    writeBatchMerge(key: string, value: string, cf_name?: string | null): Promise<string | null | undefined>;
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
    writeBatchDelete(key: string, cf_name?: string | null): Promise<string | null | undefined>;
    /**
     * Writes the current write batch to the database.
     * This function handles the `write_batch_write` action which writes the current write batch to the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    writeBatchWrite(): Promise<string | null | undefined>;
    /**
     * Clears the current write batch.
     * This function handles the `write_batch_clear` action which clears the current write batch.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    writeBatchClear(): Promise<string | null | undefined>;
    /**
     * Destroys the current write batch.
     * This function handles the `write_batch_destroy` action which destroys the current write batch.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    writeBatchDestroy(): Promise<string | null | undefined>;
    /**
     * Creates a new iterator for the database.
     * This function handles the `create_iterator` action which creates a new iterator for iterating over the keys in the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    createIterator(): Promise<string | null | undefined>;
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
    destroyIterator(iterator_id: string): Promise<string | null | undefined>;
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
    iteratorSeek(iterator_id: string, key: string): Promise<string | null | undefined>;
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
    iteratorNext(iterator_id: string): Promise<string | null | undefined>;
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
    iteratorPrev(iterator_id: string): Promise<string | null | undefined>;
    /**
     * Creates a backup of the database.
     * This function handles the `backup` action which creates a backup of the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    backup(): Promise<string | null | undefined>;
    /**
     * Restores the database from the latest backup.
     * This function handles the `restore_latest` action which restores the RocksDB database from the latest backup.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    restoreLatest(): Promise<string | null | undefined>;
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
    restore(backup_id: string): Promise<string | null | undefined>;
    /**
     * Retrieves information about all backups.
     * This function handles the `get_backup_info` action which retrieves information about all backups of the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    getBackupInfo(): Promise<string | null | undefined>;
    /**
     * Begins a new transaction.
     * This function handles the `begin_transaction` action which begins a new transaction in the RocksDB database.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    beginTransaction(): Promise<string | null | undefined>;
    /**
     * Commits an existing transaction.
     * This function handles the `commit_transaction` action which commits an existing transaction in the RocksDB database.
     * The function requires the ID of the transaction to commit.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    commitTransaction(): Promise<string | null | undefined>;
    /**
     * Rolls back an existing transaction.
     * This function handles the `rollback_transaction` action which rolls back an existing transaction in the RocksDB database.
     * The function requires the ID of the transaction to roll back.
     *
     *
     * @return {Promise<any>} The result of the operation.
     * @throws {Error} If the operation fails.
     */
    rollbackTransaction(): Promise<string | null | undefined>;
}
export default RocksDBClient;
