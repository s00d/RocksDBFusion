<?php

namespace RocksDBClient;

use Exception;

class RocksDBClient {
    private string $host;
    private int $port;
    private ?string $token;
    private $socket;
    private int $timeout;
    private int $retryInterval;

    /**
     * Constructor to initialize the RocksDB client.
     *
     * @param string $host The host of the RocksDB server.
     * @param int $port The port of the RocksDB server.
     * @param string|null $token Optional authentication token for the RocksDB server.
     */
    public function __construct(string $host, int $port, string $token = null, int $timeout = 60, int retryInterval = 2) {
        $this->host = $host;
        $this->port = $port;
        $this->token = $token;
        $this->timeout = $timeout;
        $this->retryInterval = $retryInterval;
    }

    /**
     * Connects to the RocksDB server with retry mechanism.
     *
     * @throws Exception If unable to connect to the server.
     */
    private function connect() {
        $retryInterval = 5; // Interval between retries in seconds
        $startTime = time();

        while (true) {
            $this->socket = @stream_socket_client("tcp://{$this->host}:{$this->port}", $errno, $errstr, 30);

            if ($this->socket) {
                return; // Connection successful
            }

            if (time() - $startTime >= $this->timeout) {
                throw new Exception("Unable to connect to server: $errstr ($errno)");
            }

            // Wait for the retry interval before trying again
            sleep($$this->retryInterval);
        }
    }

    /**
     * Sends a request to the RocksDB server.
     *
     * @param array $request The request to be sent.
     * @return array The response from the server.
     * @throws Exception If the response from the server is invalid.
     */
    private function sendRequest(array $request): array {
        if (!$this->socket) {
            $this->connect();
        }

        if ($this->token !== null) {
            $request['token'] = $this->token; // Add token to request if present
        }

        $requestJson = json_encode($request) . "\n";
        fwrite($this->socket, $requestJson);

        $responseJson = '';
        while (!feof($this->socket)) {
            $responseJson .= fgets($this->socket);
            if (strpos($responseJson, "\n") !== false) {
                break;
            }
        }

        $response = json_decode($responseJson, true);

        if ($response === null) {
            throw new Exception("Invalid response from server");
        }

        return $response;
    }

    /**
     * Handles the response from the server.
     *
     * @param array $response The response from the server.
     * @return mixed The result from the response.
     * @throws Exception If the response indicates an error.
     */
    private function handleResponse(array $response) {
        if ($response['success']) {
            return $response['result'];
        } else {
            throw new Exception($response['error']);
        }
    }

    /**
     * Stores a key-value pair in the database.
     *
     * @param string $key The key to store.
     * @param string $value The value to store.
     * @param string|null $cfName Optional column family name.
     * @param int|null $txnId Optional transaction ID.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function put(string $key, string $value, string $cfName = null, int $txnId = null) {
        $value = str_replace(["\r", "\n"], '', $value);
        $request = [
            'action' => 'put',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName,
        ];

        if ($txnId !== null) {
            $request['txn_id'] = $txnId;
        }

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Retrieves the value of a key from the database.
     *
     * @param string $key The key to retrieve.
     * @param string|null $cfName Optional column family name.
     * @param string|null $default Optional default value if the key is not found.
     * @param int|null $txnId Optional transaction ID.
     * @return mixed The value of the key.
     * @throws Exception If the operation fails.
     */
    public function get(string $key, string $cfName = null, string $default = null, int $txnId = null) {
        $request = [
            'action' => 'get',
            'key' => $key,
            'cf_name' => $cfName,
            'default' => $default,
        ];

        if ($txnId !== null) {
            $request['txn_id'] = $txnId;
        }

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Deletes a key from the database.
     *
     * @param string $key The key to delete.
     * @param string|null $cfName Optional column family name.
     * @param int|null $txnId Optional transaction ID.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function delete(string $key, string $cfName = null, int $txnId = null) {
        $request = [
            'action' => 'delete',
            'key' => $key,
            'cf_name' => $cfName,
        ];

        if ($txnId !== null) {
            $request['txn_id'] = $txnId;
        }

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Merges a value with an existing key.
     *
     * @param string $key The key to merge.
     * @param string $value The value to merge.
     * @param string|null $cfName Optional column family name.
     * @param int|null $txnId Optional transaction ID.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function merge(string $key, string $value, string $cfName = null, int $txnId = null) {
        $request = [
            'action' => 'merge',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName,
        ];

        if ($txnId !== null) {
            $request['txn_id'] = $txnId;
        }

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Lists all column families in the database.
     *
     * @param string $path The path to the database.
     * @return mixed The list of column families.
     * @throws Exception If the operation fails.
     */
    public function listColumnFamilies(string $path) {
        $request = [
            'action' => 'list_column_families',
            'value' => $path,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Creates a new column family.
     *
     * @param string $cfName The name of the new column family.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function createColumnFamily(string $cfName) {
        $request = [
            'action' => 'create_column_family',
            'value' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Drops an existing column family.
     *
     * @param string $cfName The name of the column family to drop.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function dropColumnFamily(string $cfName) {
        $request = [
            'action' => 'drop_column_family',
            'value' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Compacts the database within a range.
     *
     * @param string|null $start The start key of the range.
     * @param string|null $end The end key of the range.
     * @param string|null $cfName Optional column family name.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function compactRange(string $start = null, string $end = null, string $cfName = null) {
        $request = [
            'action' => 'compact_range',
            'key' => $start,
            'value' => $end,
            'cf_name' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Adds a put operation to the write batch.
     *
     * @param string $key The key to put.
     * @param string $value The value to put.
     * @param string|null $cfName Optional column family name.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function writeBatchPut(string $key, string $value, string $cfName = null) {
        $request = [
            'action' => 'write_batch_put',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Adds a merge operation to the write batch.
     *
     * @param string $key The key to merge.
     * @param string $value The value to merge.
     * @param string|null $cfName Optional column family name.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function writeBatchMerge(string $key, string $value, string $cfName = null) {
        $request = [
            'action' => 'write_batch_merge',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Adds a delete operation to the write batch.
     *
     * @param string $key The key to delete.
     * @param string|null $cfName Optional column family name.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function writeBatchDelete(string $key, string $cfName = null) {
        $request = [
            'action' => 'write_batch_delete',
            'key' => $key,
            'cf_name' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Executes all operations in the write batch.
     *
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function writeBatchWrite() {
        $request = ['action' => 'write_batch_write'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Clears all operations in the write batch.
     *
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function writeBatchClear() {
        $request = ['action' => 'write_batch_clear'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Destroys the current write batch.
     *
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function writeBatchDestroy() {
        $request = ['action' => 'write_batch_destroy'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Creates a new iterator.
     *
     * @return mixed The iterator ID.
     * @throws Exception If the operation fails.
     */
    public function createIterator() {
        $request = ['action' => 'create_iterator'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Destroys an existing iterator.
     *
     * @param int $iteratorId The ID of the iterator to destroy.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function destroyIterator(int $iteratorId) {
        $request = [
            'action' => 'destroy_iterator',
            'iterator_id' => $iteratorId,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Seeks an iterator to a specific key.
     *
     * @param int $iteratorId The ID of the iterator.
     * @param string $key The key to seek to.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function iteratorSeek(int $iteratorId, string $key) {
        $request = [
            'action' => 'iterator_seek',
            'iterator_id' => $iteratorId,
            'key' => $key,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Seeks an iterator to a specific key for the previous operation.
     *
     * @param int $iteratorId The ID of the iterator.
     * @param string $key The key to seek to.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function iteratorSeekForPrev(int $iteratorId, string $key) {
        $request = [
            'action' => 'iterator_seek_for_prev',
            'iterator_id' => $iteratorId,
            'key' => $key,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Moves an iterator to the next key-value pair.
     *
     * @param int $iteratorId The ID of the iterator.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function iteratorNext(int $iteratorId) {
        $request = [
            'action' => 'iterator_next',
            'iterator_id' => $iteratorId,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Moves an iterator to the previous key-value pair.
     *
     * @param int $iteratorId The ID of the iterator.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function iteratorPrev(int $iteratorId) {
        $request = [
            'action' => 'iterator_prev',
            'iterator_id' => $iteratorId,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Retrieves a list of keys from the database.
     *
     * @param int $start The starting index of the keys.
     * @param int $limit The maximum number of keys to retrieve.
     * @param string|null $query Optional query to filter keys.
     * @return mixed The list of keys.
     * @throws Exception If the operation fails.
     */
    public function getKeys(int $start = 0, int $limit = 20, string $query = null) {
        $request = [
            'action' => 'keys',
            'options' => [
                'start' => $start,
                'limit' => $limit,
                'query' => $query,
            ],
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Retrieves all keys from the database.
     *
     * @param string|null $query Optional query to filter keys.
     * @return mixed The list of keys.
     * @throws Exception If the operation fails.
     */
    public function getAll(string $query = null) {
        $request = [
            'action' => 'all',
            'options' => [
                'query' => $query,
            ],
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Begins a new transaction.
     *
     * @return mixed The transaction ID.
     * @throws Exception If the operation fails.
     */
    public function beginTransaction() {
        $request = ['action' => 'begin_transaction'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Commits an existing transaction.
     *
     * @param int $txnId The ID of the transaction to commit.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function commitTransaction(int $txnId) {
        $request = [
            'action' => 'commit_transaction',
            'txn_id' => $txnId,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Rolls back an existing transaction.
     *
     * @param int $txnId The ID of the transaction to roll back.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function rollbackTransaction(int $txnId) {
        $request = [
            'action' => 'rollback_transaction',
            'txn_id' => $txnId,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Retrieves a property from the database.
     *
     * @param string $property The property to retrieve.
     * @param string|null $cfName Optional column family name.
     * @return mixed The value of the property.
     * @throws Exception If the operation fails.
     */
    public function getProperty(string $property, string $cfName = null) {
        $request = [
            'action' => 'get_property',
            'value' => $property,
            'cf_name' => $cfName,
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Creates a backup of the database.
     *
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function backup() {
        $request = ['action' => 'backup'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Restores the database from the latest backup.
     *
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function restoreLatest() {
        $request = ['action' => 'restore_latest'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Restores the database from a specific backup.
     *
     * @param int $backupId The ID of the backup to restore.
     * @return mixed The result of the operation.
     * @throws Exception If the operation fails.
     */
    public function restore(int $backupId) {
        $request = [
            'action' => 'restore',
            'options' => [
                'backup_id' => $backupId,
            ],
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    /**
     * Retrieves information about the backups.
     *
     * @return mixed The list of backups.
     * @throws Exception If the operation fails.
     */
    public function getBackupInfo() {
        $request = ['action' => 'get_backup_info'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }
}
