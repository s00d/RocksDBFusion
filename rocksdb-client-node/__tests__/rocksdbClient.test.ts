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
