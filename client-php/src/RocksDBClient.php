<?php

namespace RocksDBClient;

use Exception;

class RocksDBClient {
    private $host;
    private $port;
    private $token;
    private $socket;

    public function __construct(string $host, int $port, string $token = null) {
        $this->host = $host;
        $this->port = $port;
        $this->token = $token;
    }

    private function connect() {
        $timeout = 60; // Total timeout in seconds
            $retryInterval = 5; // Interval between retries in seconds
            $startTime = time();

            while (true) {
                $this->socket = @stream_socket_client("tcp://{$this->host}:{$this->port}", $errno, $errstr, 30);

                if ($this->socket) {
                    return; // Connection successful
                }

                if (time() - $startTime >= $timeout) {
                    throw new Exception("Unable to connect to server: $errstr ($errno)");
                }

                // Wait for the retry interval before trying again
                sleep($retryInterval);
            }
    }

    private function sendRequest($request) {
        if (!$this->socket) {
            $this->connect();
        }

        if ($this->token !== null) {
            $request['token'] = $this->token; // Добавляем токен в запрос, если он есть
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


    private function handleResponse($response) {
        if ($response['success']) {
            return $response['result'];
        } else {
            throw new Exception($response['error']);
        }
    }

    public function put($key, $value, $cfName = null) {
        $value = str_replace(["\r", "\n"], '', $value);
        $request = [
            'action' => 'put',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function get($key, $cfName = null, string $default = null) {
        $request = [
            'action' => 'get',
            'key' => $key,
            'cf_name' => $cfName,
            'default' => $default
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function delete($key, $cfName = null) {
        $request = [
            'action' => 'delete',
            'key' => $key,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function merge($key, $value, $cfName = null) {
        $request = [
            'action' => 'merge',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function listColumnFamilies($path) {
        $request = [
            'action' => 'list_column_families',
            'value' => $path
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function createColumnFamily($cfName) {
        $request = [
            'action' => 'create_column_family',
            'value' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function dropColumnFamily($cfName) {
        $request = [
            'action' => 'drop_column_family',
            'value' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function compactRange($start = null, $end = null, $cfName = null) {
        $request = [
            'action' => 'compact_range',
            'key' => $start,
            'value' => $end,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function writeBatchPut($key, $value, $cfName = null) {
        $request = [
            'action' => 'write_batch_put',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function writeBatchMerge($key, $value, $cfName = null) {
        $request = [
            'action' => 'write_batch_merge',
            'key' => $key,
            'value' => $value,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function writeBatchDelete($key, $cfName = null) {
        $request = [
            'action' => 'write_batch_delete',
            'key' => $key,
            'cf_name' => $cfName
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function writeBatchWrite() {
        $request = ['action' => 'write_batch_write'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function writeBatchClear() {
        $request = ['action' => 'write_batch_clear'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function writeBatchDestroy() {
        $request = ['action' => 'write_batch_destroy'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function createIterator() {
        $request = ['action' => 'create_iterator'];
        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function destroyIterator($iteratorId) {
        $request = [
            'action' => 'destroy_iterator',
            'iterator_id' => $iteratorId
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function iteratorSeek($iteratorId, $key) {
        $request = [
            'action' => 'iterator_seek',
            'iterator_id' => $iteratorId,
            'key' => $key
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function iteratorSeekForPrev($iteratorId, $key) {
        $request = [
            'action' => 'iterator_seek_for_prev',
            'iterator_id' => $iteratorId,
            'key' => $key
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function iteratorNext($iteratorId) {
        $request = [
            'action' => 'iterator_next',
            'iterator_id' => $iteratorId
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }

    public function iteratorPrev($iteratorId) {
        $request = [
            'action' => 'iterator_prev',
            'iterator_id' => $iteratorId
        ];

        $response = $this->sendRequest($request);
        return $this->handleResponse($response);
    }
}
