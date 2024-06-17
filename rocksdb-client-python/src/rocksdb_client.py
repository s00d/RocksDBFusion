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

        request_json = json.dumps(request) + "\n"
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
            if b"\n" in data:
                break
        return data.decode('utf-8')

    def handle_response(self, response):
        if response['success'] and 'result' in response:
            return response['result']
        raise Exception(response['error'])

        async def put(self, key: str, value: str, cf_name: str|None = None, txn: bool|None = None):
        """
        Inserts a key-value pair into the database.
        This function handles the `put` action which inserts a specified key-value pair into the RocksDB database.
        The function can optionally operate within a specified column family and transaction if provided.
        @param  key: The key to put
        @param  value: The value to put
        @param  cf_name: The column family name
        @param  txn: The transaction ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "put",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key
        if "options." in "value":
            request["options"]["value"] = value
        else:
            request["value"] = value

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name
        if txn is not None:
            if "options." in "txn":
                request["options"]["txn"] = txn
            else:
                request["txn"] = txn

        response = await self.send_request(request)
        return self.handle_response(response)

    async def get(self, key: str, cf_name: str|None = None, default_value: str|None = None, txn: bool|None = None):
        """
        Retrieves the value associated with a key from the database.
        This function handles the `get` action which fetches the value associated with a specified key from the RocksDB database.
        The function can optionally operate within a specified column family and return a default value if the key is not found.
        @param  key: The key to get
        @param  cf_name: The column family name
        @param  default_value: The default value
        @param  txn: The transaction ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "get",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name
        if default_value is not None:
            if "options." in "default_value":
                request["options"]["default_value"] = default_value
            else:
                request["default_value"] = default_value
        if txn is not None:
            if "options." in "txn":
                request["options"]["txn"] = txn
            else:
                request["txn"] = txn

        response = await self.send_request(request)
        return self.handle_response(response)

    async def delete(self, key: str, cf_name: str|None = None, txn: bool|None = None):
        """
        Deletes a key-value pair from the database.
        This function handles the `delete` action which removes a specified key-value pair from the RocksDB database.
        The function can optionally operate within a specified column family and transaction if provided.
        @param  key: The key to delete
        @param  cf_name: The column family name
        @param  txn: The transaction ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "delete",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name
        if txn is not None:
            if "options." in "txn":
                request["options"]["txn"] = txn
            else:
                request["txn"] = txn

        response = await self.send_request(request)
        return self.handle_response(response)

    async def merge(self, key: str, value: str, cf_name: str|None = None, txn: bool|None = None):
        """
        Merges a value with an existing key in the database.
        This function handles the `merge` action which merges a specified value with an existing key in the RocksDB database.
        The function can optionally operate within a specified column family and transaction if provided.
        @param  key: The key to merge
        @param  value: The value to merge
        @param  cf_name: The column family name
        @param  txn: The transaction ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "merge",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key
        if "options." in "value":
            request["options"]["value"] = value
        else:
            request["value"] = value

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name
        if txn is not None:
            if "options." in "txn":
                request["options"]["txn"] = txn
            else:
                request["txn"] = txn

        response = await self.send_request(request)
        return self.handle_response(response)

    async def get_property(self, value: str, cf_name: str|None = None):
        """
        Retrieves a property of the database.
        This function handles the `get_property` action which fetches a specified property of the RocksDB database.
        The function can optionally operate within a specified column family if provided.
        @param  value: The property to get
        @param  cf_name: The column family name
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "get_property",
            "options": {},
        }

        if "options." in "value":
            request["options"]["value"] = value
        else:
            request["value"] = value

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name

        response = await self.send_request(request)
        return self.handle_response(response)

    async def keys(self, start: str, limit: str, query: str|None = None):
        """
        Retrieves a range of keys from the database.
        This function handles the `keys` action which retrieves a range of keys from the RocksDB database.
        The function can specify a starting index, limit on the number of keys, and a query string to filter keys.
        @param  start: The start index
        @param  limit: The limit of keys to retrieve
        @param  query: The query string to filter keys
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "keys",
            "options": {},
        }

        if "options." in "options.start":
            request["options"]["start"] = start
        else:
            request["options.start"] = start
        if "options." in "options.limit":
            request["options"]["limit"] = limit
        else:
            request["options.limit"] = limit

        if query is not None:
            if "options." in "options.query":
                request["options"]["query"] = query
            else:
                request["options.query"] = query

        response = await self.send_request(request)
        return self.handle_response(response)

    async def all(self, query: str|None = None):
        """
        Retrieves all keys from the database.
        This function handles the `all` action which retrieves all keys from the RocksDB database.
        The function can specify a query string to filter keys.
        @param  query: The query string to filter keys
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "all",
            "options": {},
        }


        if query is not None:
            if "options." in "options.query":
                request["options"]["query"] = query
            else:
                request["options.query"] = query

        response = await self.send_request(request)
        return self.handle_response(response)

    async def list_column_families(self, ):
        """
        Lists all column families in the database.
        This function handles the `list_column_families` action which lists all column families in the RocksDB database.
        The function requires the path to the database.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "list_column_families",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def create_column_family(self, cf_name: str):
        """
        Creates a new column family in the database.
        This function handles the `create_column_family` action which creates a new column family in the RocksDB database.
        The function requires the name of the column family to create.
        @param  cf_name: The column family name to create
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "create_column_family",
            "options": {},
        }

        if "options." in "cf_name":
            request["options"]["cf_name"] = cf_name
        else:
            request["cf_name"] = cf_name


        response = await self.send_request(request)
        return self.handle_response(response)

    async def drop_column_family(self, cf_name: str):
        """
        Drops an existing column family from the database.
        This function handles the `drop_column_family` action which drops an existing column family from the RocksDB database.
        The function requires the name of the column family to drop.
        @param  cf_name: The column family name to drop
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "drop_column_family",
            "options": {},
        }

        if "options." in "cf_name":
            request["options"]["cf_name"] = cf_name
        else:
            request["cf_name"] = cf_name


        response = await self.send_request(request)
        return self.handle_response(response)

    async def compact_range(self, start: str|None = None, end: str|None = None, cf_name: str|None = None):
        """
        Compacts a range of keys in the database.
        This function handles the `compact_range` action which compacts a specified range of keys in the RocksDB database.
        The function can optionally specify the start key, end key, and column family.
        @param  start: The start key
        @param  end: The end key
        @param  cf_name: The column family name
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "compact_range",
            "options": {},
        }


        if start is not None:
            if "options." in "options.start":
                request["options"]["start"] = start
            else:
                request["options.start"] = start
        if end is not None:
            if "options." in "options.end":
                request["options"]["end"] = end
            else:
                request["options.end"] = end
        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name

        response = await self.send_request(request)
        return self.handle_response(response)

    async def write_batch_put(self, key: str, value: str, cf_name: str|None = None):
        """
        Adds a key-value pair to the current write batch.
        This function handles the `write_batch_put` action which adds a specified key-value pair to the current write batch.
        The function can optionally operate within a specified column family.
        @param  key: The key to put
        @param  value: The value to put
        @param  cf_name: The column family name
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "write_batch_put",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key
        if "options." in "value":
            request["options"]["value"] = value
        else:
            request["value"] = value

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name

        response = await self.send_request(request)
        return self.handle_response(response)

    async def write_batch_merge(self, key: str, value: str, cf_name: str|None = None):
        """
        Merges a value with an existing key in the current write batch.
        This function handles the `write_batch_merge` action which merges a specified value with an existing key in the current write batch.
        The function can optionally operate within a specified column family.
        @param  key: The key to merge
        @param  value: The value to merge
        @param  cf_name: The column family name
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "write_batch_merge",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key
        if "options." in "value":
            request["options"]["value"] = value
        else:
            request["value"] = value

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name

        response = await self.send_request(request)
        return self.handle_response(response)

    async def write_batch_delete(self, key: str, cf_name: str|None = None):
        """
        Deletes a key from the current write batch.
        This function handles the `write_batch_delete` action which deletes a specified key from the current write batch.
        The function can optionally operate within a specified column family.
        @param  key: The key to delete
        @param  cf_name: The column family name
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "write_batch_delete",
            "options": {},
        }

        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key

        if cf_name is not None:
            if "options." in "cf_name":
                request["options"]["cf_name"] = cf_name
            else:
                request["cf_name"] = cf_name

        response = await self.send_request(request)
        return self.handle_response(response)

    async def write_batch_write(self, ):
        """
        Writes the current write batch to the database.
        This function handles the `write_batch_write` action which writes the current write batch to the RocksDB database.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "write_batch_write",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def write_batch_clear(self, ):
        """
        Clears the current write batch.
        This function handles the `write_batch_clear` action which clears the current write batch.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "write_batch_clear",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def write_batch_destroy(self, ):
        """
        Destroys the current write batch.
        This function handles the `write_batch_destroy` action which destroys the current write batch.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "write_batch_destroy",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def create_iterator(self, ):
        """
        Creates a new iterator for the database.
        This function handles the `create_iterator` action which creates a new iterator for iterating over the keys in the RocksDB database.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "create_iterator",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def destroy_iterator(self, iterator_id: str):
        """
        Destroys an existing iterator.
        This function handles the `destroy_iterator` action which destroys an existing iterator in the RocksDB database.
        The function requires the ID of the iterator to destroy.
        @param  iterator_id: The iterator ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "destroy_iterator",
            "options": {},
        }

        if "options." in "options.iterator_id":
            request["options"]["iterator_id"] = iterator_id
        else:
            request["options.iterator_id"] = iterator_id


        response = await self.send_request(request)
        return self.handle_response(response)

    async def iterator_seek(self, iterator_id: str, key: str):
        """
        Seeks to a specific key in the iterator.
        This function handles the `iterator_seek` action which seeks to a specified key in an existing iterator in the RocksDB database.
        The function requires the ID of the iterator, the key to seek, and the direction of the seek (Forward or Reverse).
        @param  iterator_id: The iterator ID
        @param  key: The key to seek
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "iterator_seek",
            "options": {},
        }

        if "options." in "options.iterator_id":
            request["options"]["iterator_id"] = iterator_id
        else:
            request["options.iterator_id"] = iterator_id
        if "options." in "key":
            request["options"]["key"] = key
        else:
            request["key"] = key


        response = await self.send_request(request)
        return self.handle_response(response)

    async def iterator_next(self, iterator_id: str):
        """
        Advances the iterator to the next key.
        This function handles the `iterator_next` action which advances an existing iterator to the next key in the RocksDB database.
        The function requires the ID of the iterator.
        @param  iterator_id: The iterator ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "iterator_next",
            "options": {},
        }

        if "options." in "options.iterator_id":
            request["options"]["iterator_id"] = iterator_id
        else:
            request["options.iterator_id"] = iterator_id


        response = await self.send_request(request)
        return self.handle_response(response)

    async def iterator_prev(self, iterator_id: str):
        """
        Moves the iterator to the previous key.
        This function handles the `iterator_prev` action which moves an existing iterator to the previous key in the RocksDB database.
        The function requires the ID of the iterator.
        @param  iterator_id: The iterator ID
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "iterator_prev",
            "options": {},
        }

        if "options." in "options.iterator_id":
            request["options"]["iterator_id"] = iterator_id
        else:
            request["options.iterator_id"] = iterator_id


        response = await self.send_request(request)
        return self.handle_response(response)

    async def backup(self, ):
        """
        Creates a backup of the database.
        This function handles the `backup` action which creates a backup of the RocksDB database.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "backup",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def restore_latest(self, ):
        """
        Restores the database from the latest backup.
        This function handles the `restore_latest` action which restores the RocksDB database from the latest backup.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "restore_latest",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def restore(self, backup_id: str):
        """
        Restores the database from a specified backup.
        This function handles the `restore` action which restores the RocksDB database from a specified backup.
        The function requires the ID of the backup to restore.
        @param  backup_id: The ID of the backup to restore
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "restore",
            "options": {},
        }

        if "options." in "options.backup_id":
            request["options"]["backup_id"] = backup_id
        else:
            request["options.backup_id"] = backup_id


        response = await self.send_request(request)
        return self.handle_response(response)

    async def get_backup_info(self, ):
        """
        Retrieves information about all backups.
        This function handles the `get_backup_info` action which retrieves information about all backups of the RocksDB database.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "get_backup_info",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def begin_transaction(self, ):
        """
        Begins a new transaction.
        This function handles the `begin_transaction` action which begins a new transaction in the RocksDB database.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "begin_transaction",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def commit_transaction(self, ):
        """
        Commits an existing transaction.
        This function handles the `commit_transaction` action which commits an existing transaction in the RocksDB database.
        The function requires the ID of the transaction to commit.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "commit_transaction",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)

    async def rollback_transaction(self, ):
        """
        Rolls back an existing transaction.
        This function handles the `rollback_transaction` action which rolls back an existing transaction in the RocksDB database.
        The function requires the ID of the transaction to roll back.
        @return: The result of the operation.
        @rtype: Any
        @raises Exception: If the operation fails.
        """
        request = {
            "action": "rollback_transaction",
            "options": {},
        }



        response = await self.send_request(request)
        return self.handle_response(response)


