package rocksdbclient

import (
	"bufio"
	"encoding/json"
	"fmt"
	"net"
	"time"
)

type Request struct {
	Action       string            `json:"action"`
	Key          *string           `json:"key,omitempty"`
	Value        *string           `json:"value,omitempty"`
	CfName       *string           `json:"cf_name,omitempty"`
	DefaultValue *string           `json:"default_value,omitempty"`
	Options      map[string]string `json:"options,omitempty"`
	Token        *string           `json:"token,omitempty"`
	Txn          *bool             `json:"txn,omitempty"`
}

type Response struct {
	Success bool   `json:"success"`
	Result  string `json:"result"`
}

type RocksDBClient struct {
	host          string
	port          int
	token         *string
	timeout       time.Duration
	retryInterval time.Duration
	conn          net.Conn
}

func NewRocksDBClient(host string, port int, token *string, timeout, retryInterval time.Duration) *RocksDBClient {
	return &RocksDBClient{
		host:          host,
		port:          port,
		token:         token,
		timeout:       timeout,
		retryInterval: retryInterval,
	}
}

func (c *RocksDBClient) Connect() error {
	start := time.Now()
	for {
		conn, err := net.DialTimeout("tcp", fmt.Sprintf("%s:%d", c.host, c.port), c.timeout)
		if err == nil {
			c.conn = conn
			return nil
		}
		if time.Since(start) >= c.timeout {
			return fmt.Errorf("unable to connect to server: %w", err)
		}
		time.Sleep(c.retryInterval)
	}
}

func (c *RocksDBClient) Close() {
	if c.conn != nil {
		c.conn.Close()
		c.conn = nil
	}
}

func (c *RocksDBClient) SendRequest(request Request) (*Response, error) {
	if c.conn == nil {
		if err := c.Connect(); err != nil {
			return nil, err
		}
	}

	if c.token != nil {
		request.Token = c.token
	}

	encoder := json.NewEncoder(c.conn)
	if err := encoder.Encode(request); err != nil {
		return nil, fmt.Errorf("error sending request: %w", err)
	}

	response := &Response{}
	decoder := json.NewDecoder(bufio.NewReader(c.conn))
	if err := decoder.Decode(response); err != nil {
		return nil, fmt.Errorf("error decoding response: %w", err)
	}

	if !response.Success {
		return nil, fmt.Errorf("server error: %s", response.Result)
	}

	return response, nil
}

/**
* Inserts a key-value pair into the database.
    * This function handles the `put` action which inserts a specified key-value pair into the RocksDB database.
    * The function can optionally operate within a specified column family and transaction if provided.
*
* @param string Key The key to put
* @param string Value The value to put
* @param string CfName The column family name
* @param bool Txn The transaction ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Put(Key *string, Value *string, CfName *string, Txn *bool) (*Response, error) {
	request := Request{
		Action:  "put",
		Options: map[string]string{},
	}

	request.Key = Key
	request.Value = Value

	request.CfName = CfName
	request.Txn = Txn

	return c.SendRequest(request)
}

/**
* Retrieves the value associated with a key from the database.
    * This function handles the `get` action which fetches the value associated with a specified key from the RocksDB database.
    * The function can optionally operate within a specified column family and return a default value if the key is not found.
*
* @param string Key The key to get
* @param string CfName The column family name
* @param string DefaultValue The default value
* @param bool Txn The transaction ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Get(Key *string, CfName *string, DefaultValue *string, Txn *bool) (*Response, error) {
	request := Request{
		Action:  "get",
		Options: map[string]string{},
	}

	request.Key = Key

	request.CfName = CfName
	request.DefaultValue = DefaultValue
	request.Txn = Txn

	return c.SendRequest(request)
}

/**
* Deletes a key-value pair from the database.
    * This function handles the `delete` action which removes a specified key-value pair from the RocksDB database.
    * The function can optionally operate within a specified column family and transaction if provided.
*
* @param string Key The key to delete
* @param string CfName The column family name
* @param bool Txn The transaction ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Delete(Key *string, CfName *string, Txn *bool) (*Response, error) {
	request := Request{
		Action:  "delete",
		Options: map[string]string{},
	}

	request.Key = Key

	request.CfName = CfName
	request.Txn = Txn

	return c.SendRequest(request)
}

/**
* Merges a value with an existing key in the database.
    * This function handles the `merge` action which merges a specified value with an existing key in the RocksDB database.
    * The function can optionally operate within a specified column family and transaction if provided.
*
* @param string Key The key to merge
* @param string Value The value to merge
* @param string CfName The column family name
* @param bool Txn The transaction ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Merge(Key *string, Value *string, CfName *string, Txn *bool) (*Response, error) {
	request := Request{
		Action:  "merge",
		Options: map[string]string{},
	}

	request.Key = Key
	request.Value = Value

	request.CfName = CfName
	request.Txn = Txn

	return c.SendRequest(request)
}

/**
* Retrieves a property of the database.
    * This function handles the `get_property` action which fetches a specified property of the RocksDB database.
    * The function can optionally operate within a specified column family if provided.
*
* @param string Value The property to get
* @param string CfName The column family name
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) GetProperty(Value *string, CfName *string) (*Response, error) {
	request := Request{
		Action:  "get_property",
		Options: map[string]string{},
	}

	request.Value = Value

	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Retrieves a range of keys from the database.
    * This function handles the `keys` action which retrieves a range of keys from the RocksDB database.
    * The function can specify a starting index, limit on the number of keys, and a query string to filter keys.
*
* @param string OptionsStart The start index
* @param string OptionsLimit The limit of keys to retrieve
* @param string OptionsQuery The query string to filter keys
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Keys(OptionsStart string, OptionsLimit string, OptionsQuery string) (*Response, error) {
	request := Request{
		Action:  "keys",
		Options: map[string]string{},
	}

	request.Options["OptionsStart"] = OptionsStart
	request.Options["OptionsLimit"] = OptionsLimit

	request.Options["OptionsQuery"] = OptionsQuery

	return c.SendRequest(request)
}

/**
* Retrieves all keys from the database.
    * This function handles the `all` action which retrieves all keys from the RocksDB database.
    * The function can specify a query string to filter keys.
*
* @param string OptionsQuery The query string to filter keys
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) All(OptionsQuery string) (*Response, error) {
	request := Request{
		Action:  "all",
		Options: map[string]string{},
	}

	request.Options["OptionsQuery"] = OptionsQuery

	return c.SendRequest(request)
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
func (c *RocksDBClient) ListColumnFamilies() (*Response, error) {
	request := Request{
		Action:  "list_column_families",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Creates a new column family in the database.
    * This function handles the `create_column_family` action which creates a new column family in the RocksDB database.
    * The function requires the name of the column family to create.
*
* @param string CfName The column family name to create
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) CreateColumnFamily(CfName *string) (*Response, error) {
	request := Request{
		Action:  "create_column_family",
		Options: map[string]string{},
	}

	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Drops an existing column family from the database.
    * This function handles the `drop_column_family` action which drops an existing column family from the RocksDB database.
    * The function requires the name of the column family to drop.
*
* @param string CfName The column family name to drop
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) DropColumnFamily(CfName *string) (*Response, error) {
	request := Request{
		Action:  "drop_column_family",
		Options: map[string]string{},
	}

	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Compacts a range of keys in the database.
    * This function handles the `compact_range` action which compacts a specified range of keys in the RocksDB database.
    * The function can optionally specify the start key, end key, and column family.
*
* @param string OptionsStart The start key
* @param string OptionsEnd The end key
* @param string CfName The column family name
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) CompactRange(OptionsStart string, OptionsEnd string, CfName *string) (*Response, error) {
	request := Request{
		Action:  "compact_range",
		Options: map[string]string{},
	}

	request.Options["OptionsStart"] = OptionsStart
	request.Options["OptionsEnd"] = OptionsEnd
	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Adds a key-value pair to the current write batch.
    * This function handles the `write_batch_put` action which adds a specified key-value pair to the current write batch.
    * The function can optionally operate within a specified column family.
*
* @param string Key The key to put
* @param string Value The value to put
* @param string CfName The column family name
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) WriteBatchPut(Key *string, Value *string, CfName *string) (*Response, error) {
	request := Request{
		Action:  "write_batch_put",
		Options: map[string]string{},
	}

	request.Key = Key
	request.Value = Value

	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Merges a value with an existing key in the current write batch.
    * This function handles the `write_batch_merge` action which merges a specified value with an existing key in the current write batch.
    * The function can optionally operate within a specified column family.
*
* @param string Key The key to merge
* @param string Value The value to merge
* @param string CfName The column family name
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) WriteBatchMerge(Key *string, Value *string, CfName *string) (*Response, error) {
	request := Request{
		Action:  "write_batch_merge",
		Options: map[string]string{},
	}

	request.Key = Key
	request.Value = Value

	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Deletes a key from the current write batch.
    * This function handles the `write_batch_delete` action which deletes a specified key from the current write batch.
    * The function can optionally operate within a specified column family.
*
* @param string Key The key to delete
* @param string CfName The column family name
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) WriteBatchDelete(Key *string, CfName *string) (*Response, error) {
	request := Request{
		Action:  "write_batch_delete",
		Options: map[string]string{},
	}

	request.Key = Key

	request.CfName = CfName

	return c.SendRequest(request)
}

/**
* Writes the current write batch to the database.
    * This function handles the `write_batch_write` action which writes the current write batch to the RocksDB database.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) WriteBatchWrite() (*Response, error) {
	request := Request{
		Action:  "write_batch_write",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Clears the current write batch.
    * This function handles the `write_batch_clear` action which clears the current write batch.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) WriteBatchClear() (*Response, error) {
	request := Request{
		Action:  "write_batch_clear",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Destroys the current write batch.
    * This function handles the `write_batch_destroy` action which destroys the current write batch.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) WriteBatchDestroy() (*Response, error) {
	request := Request{
		Action:  "write_batch_destroy",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Creates a new iterator for the database.
    * This function handles the `create_iterator` action which creates a new iterator for iterating over the keys in the RocksDB database.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) CreateIterator() (*Response, error) {
	request := Request{
		Action:  "create_iterator",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Destroys an existing iterator.
    * This function handles the `destroy_iterator` action which destroys an existing iterator in the RocksDB database.
    * The function requires the ID of the iterator to destroy.
*
* @param string OptionsIteratorId The iterator ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) DestroyIterator(OptionsIteratorId string) (*Response, error) {
	request := Request{
		Action:  "destroy_iterator",
		Options: map[string]string{},
	}

	request.Options["OptionsIteratorId"] = OptionsIteratorId

	return c.SendRequest(request)
}

/**
* Seeks to a specific key in the iterator.
    * This function handles the `iterator_seek` action which seeks to a specified key in an existing iterator in the RocksDB database.
    * The function requires the ID of the iterator, the key to seek, and the direction of the seek (Forward or Reverse).
*
* @param string OptionsIteratorId The iterator ID
* @param string Key The key to seek
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) IteratorSeek(OptionsIteratorId string, Key *string) (*Response, error) {
	request := Request{
		Action:  "iterator_seek",
		Options: map[string]string{},
	}

	request.Options["OptionsIteratorId"] = OptionsIteratorId
	request.Key = Key

	return c.SendRequest(request)
}

/**
* Advances the iterator to the next key.
    * This function handles the `iterator_next` action which advances an existing iterator to the next key in the RocksDB database.
    * The function requires the ID of the iterator.
*
* @param string OptionsIteratorId The iterator ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) IteratorNext(OptionsIteratorId string) (*Response, error) {
	request := Request{
		Action:  "iterator_next",
		Options: map[string]string{},
	}

	request.Options["OptionsIteratorId"] = OptionsIteratorId

	return c.SendRequest(request)
}

/**
* Moves the iterator to the previous key.
    * This function handles the `iterator_prev` action which moves an existing iterator to the previous key in the RocksDB database.
    * The function requires the ID of the iterator.
*
* @param string OptionsIteratorId The iterator ID
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) IteratorPrev(OptionsIteratorId string) (*Response, error) {
	request := Request{
		Action:  "iterator_prev",
		Options: map[string]string{},
	}

	request.Options["OptionsIteratorId"] = OptionsIteratorId

	return c.SendRequest(request)
}

/**
* Creates a backup of the database.
    * This function handles the `backup` action which creates a backup of the RocksDB database.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Backup() (*Response, error) {
	request := Request{
		Action:  "backup",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Restores the database from the latest backup.
    * This function handles the `restore_latest` action which restores the RocksDB database from the latest backup.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) RestoreLatest() (*Response, error) {
	request := Request{
		Action:  "restore_latest",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Restores the database from a specified backup.
    * This function handles the `restore` action which restores the RocksDB database from a specified backup.
    * The function requires the ID of the backup to restore.
*
* @param string OptionsBackupId The ID of the backup to restore
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) Restore(OptionsBackupId string) (*Response, error) {
	request := Request{
		Action:  "restore",
		Options: map[string]string{},
	}

	request.Options["OptionsBackupId"] = OptionsBackupId

	return c.SendRequest(request)
}

/**
* Retrieves information about all backups.
    * This function handles the `get_backup_info` action which retrieves information about all backups of the RocksDB database.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) GetBackupInfo() (*Response, error) {
	request := Request{
		Action:  "get_backup_info",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}

/**
* Begins a new transaction.
    * This function handles the `begin_transaction` action which begins a new transaction in the RocksDB database.
*
*
* @return {Promise<any>} The result of the operation.
* @throws {Error} If the operation fails.
*/
func (c *RocksDBClient) BeginTransaction() (*Response, error) {
	request := Request{
		Action:  "begin_transaction",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
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
func (c *RocksDBClient) CommitTransaction() (*Response, error) {
	request := Request{
		Action:  "commit_transaction",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
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
func (c *RocksDBClient) RollbackTransaction() (*Response, error) {
	request := Request{
		Action:  "rollback_transaction",
		Options: map[string]string{},
	}

	return c.SendRequest(request)
}
