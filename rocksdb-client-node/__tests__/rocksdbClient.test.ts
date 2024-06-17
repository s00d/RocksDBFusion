import RocksDBClient from '../src/index';

let client: RocksDBClient;

beforeEach(() => {
    client = new RocksDBClient('127.0.0.1', 12345);
});


afterEach(() => {
    client.close();
});

test('should connect to the server', async () => {
    await client.connect();
    expect(client.socket).not.toBeNull();
});

test('should put a value', async () => {
    const response = await client.put('key', 'Value');
    expect(response).toBeNull();
});

test('should get a value', async () => {
    const value = await client.get('key');
    expect(value).toBe('Value');
});

test('should delete a value', async () => {
    await client.put('key', 'Value');
    await client.delete('key');
    const value = await client.get('key', null, 'none');
    expect(value).toBe('none');
});

test('should merge a value', async () => {
    const initial_json = JSON.stringify({
        "employees": [
            { "first_name": "john", "last_name": "doe" },
            { "first_name": "adam", "last_name": "smith" }
        ]
    });

    await client.put('test_key', initial_json);

    const patch1 = JSON.stringify([
        { "op": "replace", "path": "/employees/1/first_name", "value": "lucy" }
    ]);
    await client.merge('test_key', patch1);

    const patch2 = JSON.stringify([
        { "op": "replace", "path": "/employees/0/last_name", "value": "dow" }
    ]);
    await client.merge('test_key', patch2);

    const val = await client.get('test_key') ?? '';
    const value = JSON.parse(val);

    const expected_value = {
        "employees": [
            { "first_name": "john", "last_name": "dow" },
            { "first_name": "lucy", "last_name": "smith" }
        ]
    };
    expect(value).toEqual(expected_value);

    // Добавление ключа
    const addPatch = JSON.stringify([
        { "op": "add", "path": "/employees/1/middle_name", "value": "anne" }
    ]);
    await client.merge('test_key', addPatch);

    const valAfterAdd = await client.get('test_key') ?? '';
    const valueAfterAdd = JSON.parse(valAfterAdd);

    const expectedValueAfterAdd = {
        "employees": [
            { "first_name": "john", "last_name": "dow" },
            { "first_name": "lucy", "last_name": "smith", "middle_name": "anne" }
        ]
    };
    expect(valueAfterAdd).toEqual(expectedValueAfterAdd);

    // Удаление ключа
    const removePatch = JSON.stringify([
        { "op": "remove", "path": "/employees/1/middle_name" }
    ]);
    await client.merge('test_key', removePatch);

    const valAfterRemove = await client.get('test_key') ?? '';
    const valueAfterRemove = JSON.parse(valAfterRemove);

    const expectedValueAfterRemove = {
        "employees": [
            { "first_name": "john", "last_name": "dow" },
            { "first_name": "lucy", "last_name": "smith" }
        ]
    };
    expect(valueAfterRemove).toEqual(expectedValueAfterRemove);
});

test('should create a column family', async () => {
    const response = await client.createColumnFamily('new_cf');
    expect(response).toBeNull();
});

test('should list column families', async () => {
    await client.createColumnFamily('path_to_db');

    const response = await client.listColumnFamilies();

    expect(response).toBeDefined();
});

test('should drop a column family', async () => {
    await client.createColumnFamily('cf_to_drop');
    const response = await client.dropColumnFamily('cf_to_drop');
    expect(response).toBeNull();
});

test('should put, get, delete, and merge with cf_name', async () => {
    const cf_name = 'test_cf';
    await client.createColumnFamily(cf_name);

    // Put with column family
    await client.put('cf_key', 'cf_value', cf_name);
    let value = await client.get('cf_key', cf_name);
    expect(value).toBe('cf_value');

    // Merge with column family
    value = await client.get('cf_key', cf_name);
    expect(value).toBe('cf_value');

    // Delete with column family
    await client.delete('cf_key', cf_name);
    value = await client.get('cf_key', cf_name, 'not_found');
    expect(value).toBe('not_found');

    // Cleanup
    await client.dropColumnFamily(cf_name);
});


test('should compact a range', async () => {
    const response = await client.compactRange('start_key', 'end_key');
    expect(response).toBeNull();
});

test('should handle transactions', async () => {
    const resp = await client.beginTransaction();
    expect(resp).toBeDefined();


    await client.put('txn_key', 'txn_value', null, true);
    await client.commitTransaction();

    const value = await client.get('txn_key', null);
    expect(value).toBe('txn_value');

    const resp2 = await client.beginTransaction();
    expect(resp2).toBeDefined();
    //
    await client.put('rollback_key', 'rollback_value', null, true);
    await client.rollbackTransaction();

    const rollbackValue = await client.get('rollback_key', null, 'not_found');
    expect(rollbackValue).toBe('not_found');
});

test('should create and destroy iterator', async () => {
    const iteratorId = await client.createIterator();
    expect(iteratorId).toBeDefined();

    if (!iteratorId) return;

    const destroyResponse = await client.destroyIterator(iteratorId);
    expect(destroyResponse).toBeNull();
});

test('should seek in iterator', async () => {
    const iteratorId = await client.createIterator();
    await client.put('seek_key', 'seek_value');

    if (!iteratorId) return;

    const response = await client.iteratorSeek(iteratorId, 'seek_key');
    expect(response).toBe('seek_key:seek_value');

    await client.destroyIterator(iteratorId);
});

test('should add multiple keys and retrieve using iterator', async () => {
    const numKeys = 20;
    const keys = Array.from({ length: numKeys }, (_, i) => `key${i + 1}`);
    const values = Array.from({ length: numKeys }, (_, i) => `value${i + 1}`);

    for (let i = 0; i < keys.length; i++) {
        await client.put(keys[i], values[i]);
    }

    const iteratorId = await client.createIterator();
    expect(iteratorId).toBeDefined();
    if (!iteratorId) return;

    let currentKey = await client.iteratorSeek(iteratorId, keys[0]);
    let result = [];
    while (currentKey) {
        const [key, value] = currentKey.split(':');
        if(key === 'invalid') break;
        result.push({ key, value });
        currentKey = await client.iteratorNext(iteratorId);
    }

    const expectedElements = [
        { key: 'key1', value: 'value1' },
        { key: 'key2', value: 'value2' },
        { key: 'key3', value: 'value3' },
        // Add more expected elements as needed
    ];

    for (const expectedElement of expectedElements) {
        expect(result).toEqual(expect.arrayContaining([expectedElement]));
    }

    await client.destroyIterator(iteratorId);
});

test('should handle backups', async () => {
    const backupResponse = await client.backup();
    expect(backupResponse).toBe("Backup created successfully");

    const restoreResponse = await client.restoreLatest();
    expect(restoreResponse).toBe('Database restored from latest backup');

});