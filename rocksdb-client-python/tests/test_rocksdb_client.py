import asyncio
import unittest
import json
from src.rocksdb_client import RocksDBClient

class TestRocksDBClient(unittest.TestCase):
    def test_put_get(self):
        client = RocksDBClient('127.0.0.1', 12345)
        asyncio.run(client.connect())
        asyncio.run(client.put('test_key', 'test_value'))
        value = asyncio.run(client.get('test_key'))
        self.assertEqual(value, 'test_value')
        client.close()

    def test_delete(self):
        client = RocksDBClient('127.0.0.1', 12345)
        asyncio.run(client.connect())
        asyncio.run(client.put('test_key', 'test_value'))
        asyncio.run(client.delete('test_key'))
        value = asyncio.run(client.get('test_key', None, 'default_value'))
        self.assertEqual(value, 'default_value')
        client.close()

    def test_merge(self):
        client = RocksDBClient('127.0.0.1', 12345)
        asyncio.run(client.connect())
        initial_json = json.dumps({"employees": [{"first_name": "john", "last_name": "doe"}, {"first_name": "adam", "last_name": "smith"}]})
        asyncio.run(client.put('test_key', initial_json))

        patch1 = json.dumps([{"op": "replace", "path": "/employees/1/first_name", "value": "lucy"}])
        asyncio.run(client.merge('test_key', patch1))

        patch2 = json.dumps([{"op": "replace", "path": "/employees/0/last_name", "value": "dow"}])
        asyncio.run(client.merge('test_key', patch2))

        value = json.loads(asyncio.run(client.get('test_key')))
        expected_value = {"employees": [{"first_name": "john", "last_name": "dow"}, {"first_name": "lucy", "last_name": "smith"}]}
        self.assertEqual(value, expected_value)
        client.close()

if __name__ == '__main__':
    unittest.main()
