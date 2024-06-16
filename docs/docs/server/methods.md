---
lang: en-US
sticky: 10
icon: circle-question
star: true
dir:
order: 3
category:
   - SERVER
---

# Examples

## Putting a Key-Value Pair

To put a key-value pair into the database:

```json
{
  "action": "put",
  "key": "example_key",
  "value": "example_value"
}
```

## Getting a Key-Value Pair

To get a value for a given key:

```json
{
  "action": "get",
  "key": "example_key"
}
```

## Deleting a Key-Value Pair

To delete a key-value pair:

```json
{
  "action": "delete",
  "key": "example_key"
}
```

## Merging a JSON Value

To merge a JSON value into an existing key:

```json
{
  "action": "merge",
  "key": "example_key",
  "value": "{\"new_field\": \"new_value\"}"
}
```

# Backup and Restore

## Creating a Backup

To create a backup of the database:

```json
{
  "action": "backup"
}
```

## Restoring from the Latest Backup

To restore the database from the latest backup:

```json
{
  "action": "restore_latest"
}
```

## Restoring from a Specific Backup

To restore the database from a specific backup by ID:

```json
{
  "action": "restore",
  "backup_id": 1
}
```

## Getting Backup Info

To get information about available backups:

```json
{
  "action": "get_backup_info"
}
```

# Transactions

## Beginning a Transaction

To begin a new transaction:

```json
{
  "action": "begin_transaction"
}
```

## Committing a Transaction

To commit a transaction:

```json
{
  "action": "commit_transaction",
  "txn_id": 1
}
```

## Rolling Back

Transaction

To roll back a transaction:

```json
{
  "action": "rollback_transaction",
  "txn_id": 1
}
```