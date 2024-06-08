# RocksDB Client for PHP

![Packagist Version](https://img.shields.io/packagist/v/your-username/rocksdb-client-php)
![Packagist License](https://img.shields.io/packagist/l/your-username/rocksdb-client-php)

A PHP client for interacting with a RocksDB server. This package allows you to easily perform CRUD operations, manage column families, and handle transactions and backups with a RocksDB server over TCP.

## Installation

You can install the package via Composer:

```bash
composer require your-username/rocksdb-client-php
```

## Usage

Here's a quick example to get you started:

```php
require 'vendor/autoload.php';

use RocksDBClient\RocksDBClient;

$client = new RocksDBClient('127.0.0.1', 12345);

// Put a key-value pair
$client->put('key', 'value');

// Get a value by key
$value = $client->get('key');
echo "Value: $value\n";

// Delete a key-value pair
$client->delete('key');
```

## Methods

### `put($key, $value, $cfName = null)`

Inserts a key-value pair into the database.

**Parameters:**
- `key` (string): The key to insert.
- `value` (string): The value to insert.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->put('key', 'value');
```

### `get($key, $cfName = null)`

Retrieves a value by key from the database.

**Parameters:**
- `key` (string): The key to retrieve.
- `cfName` (string, optional): The column family name.

**Returns:**
- `string|null`: The value associated with the key, or `null` if the key does not exist.

**Example:**
```php
$value = $client->get('key');
```

### `delete($key, $cfName = null)`

Deletes a key-value pair from the database.

**Parameters:**
- `key` (string): The key to delete.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->delete('key');
```

### `merge($key, $value, $cfName = null)`

Merges a value with the existing value of a key.

**Parameters:**
- `key` (string): The key to merge.
- `value` (string): The value to merge.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->merge('key', 'new_value');
```

### `listColumnFamilies($path)`

Lists all column families in the database at the specified path.

**Parameters:**
- `path` (string): The path to the database.

**Returns:**
- `array`: An array of column family names.

**Example:**
```php
$columnFamilies = $client->listColumnFamilies('/path/to/db');
```

### `createColumnFamily($cfName)`

Creates a new column family in the database.

**Parameters:**
- `cfName` (string): The name of the column family to create.

**Example:**
```php
$client->createColumnFamily('new_cf');
```

### `dropColumnFamily($cfName)`

Drops a column family from the database.

**Parameters:**
- `cfName` (string): The name of the column family to drop.

**Example:**
```php
$client->dropColumnFamily('old_cf');
```

### `compactRange($start = null, $end = null, $cfName = null)`

Compacts the database within the specified key range and column family.

**Parameters:**
- `start` (string, optional): The start key of the range.
- `end` (string, optional): The end key of the range.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->compactRange('start_key', 'end_key');
```

### Transaction Methods

#### `beginTransaction()`

Begins a new transaction.

**Example:**
```php
$client->beginTransaction();
```

#### `commitTransaction()`

Commits the current transaction.

**Example:**
```php
$client->commitTransaction();
```

#### `rollbackTransaction()`

Rolls back the current transaction.

**Example:**
```php
$client->rollbackTransaction();
```

#### `setSavepoint()`

Sets a savepoint in the current transaction.

**Example:**
```php
$client->setSavepoint();
```

#### `rollbackToSavepoint()`

Rolls back to the last savepoint in the current transaction.

**Example:**
```php
$client->rollbackToSavepoint();
```

### Backup Methods

#### `backupCreate()`

Creates a new backup.

**Example:**
```php
$client->backupCreate();
```

#### `backupInfo()`

Retrieves information about existing backups.

**Example:**
```php
$info = $client->backupInfo();
print_r($info);
```

#### `backupPurgeOld($numBackupsToKeep)`

Purges old backups, keeping only the specified number of recent backups.

**Parameters:**
- `numBackupsToKeep` (int): The number of backups to keep.

**Example:**
```php
$client->backupPurgeOld(3);
```

#### `backupRestore($backupId, $restorePath)`

Restores a backup to the specified path.

**Parameters:**
- `backupId` (int): The ID of the backup to restore.
- `restorePath` (string): The path to restore the backup to.

**Example:**
```php
$client->backupRestore(1, '/path/to/restore');
```

### Write Batch Methods

#### `writeBatchPut($key, $value, $cfName = null)`

Adds a put operation to the write batch.

**Parameters:**
- `key` (string): The key to put.
- `value` (string): The value to put.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->writeBatchPut('key', 'value');
```

#### `writeBatchMerge($key, $value, $cfName = null)`

Adds a merge operation to the write batch.

**Parameters:**
- `key` (string): The key to merge.
- `value` (string): The value to merge.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->writeBatchMerge('key', 'new_value');
```

#### `writeBatchDelete($key, $cfName = null)`

Adds a delete operation to the write batch.

**Parameters:**
- `key` (string): The key to delete.
- `cfName` (string, optional): The column family name.

**Example:**
```php
$client->writeBatchDelete('key');
```

#### `writeBatchWrite()`

Writes all operations in the write batch to the database.

**Example:**
```php
$client->writeBatchWrite();
```

#### `writeBatchClear()`

Clears all operations in the write batch.

**Example:**
```php
$client->writeBatchClear();
```

#### `writeBatchDestroy()`

Destroys the write batch.

**Example:**
```php
$client->writeBatchDestroy();
```

### Iterator Methods

#### `createIterator()`

Creates a new iterator.

**Returns:**
- `string`: The ID of the new iterator.

**Example:**
```php
$iteratorId = $client->createIterator();
```

#### `destroyIterator($iteratorId)`

Destroys the specified iterator.

**Parameters:**
- `iteratorId` (int): The ID of the iterator to destroy.

**Example:**
```php
$client->destroyIterator($iteratorId);
```

#### `iteratorSeek($iteratorId, $key)`

Seeks the specified iterator to the specified key.

**Parameters:**
- `iteratorId` (int): The ID of the iterator to seek.
- `key` (string): The key to seek to.

**Example:**
```php
$client->iteratorSeek($iteratorId, 'key');
```

#### `iteratorSeekForPrev($iteratorId, $key)`

Seeks the specified iterator to the specified key or previous.

**Parameters:**
- `iteratorId` (int): The ID of the iterator to seek.
- `key` (string): The key to seek to or previous.

**Example:**
```php
$client->iteratorSeekForPrev($iteratorId, 'key');
```

#### `iteratorNext($iteratorId)`

Moves the specified iterator to the next key.

**Parameters:**
- `iteratorId` (int): The ID of the iterator to move.

**Returns:**
- `bool`: `true` if the iterator was successfully moved, `false` otherwise.

**Example:**
```php
$client->iteratorNext($iteratorId);
```

#### `iteratorPrev($iteratorId)`

Moves the specified iterator to the previous key.

**Parameters:**
- `iteratorId` (int): The ID of the iterator to move.

**Example:**
```php
$client->iteratorPrev($iteratorId);
```

## Contributing

Contributions are welcome! Please open an issue or submit

a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.