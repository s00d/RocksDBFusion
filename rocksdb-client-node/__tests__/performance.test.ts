import RocksDBClient from 'rocksdb-client-node';
import { Bench } from 'tinybench';

jest.setTimeout(300000); // Устанавливаем тайм-аут для тестов в 300 секунд (5 минут)

let client: RocksDBClient;
const numOperations = 1000;

const numClients = 1000;
const numRequestsPerClient = 1000;

const createClient = (host: string, port: number): RocksDBClient => {
    return new RocksDBClient(host, port);
};

const performClientRequests = async (client: RocksDBClient, numRequests: number) => {
    for (let i = 0; i < numRequests; i++) {
        await client.put(`key${i}`, `value${i}`);
        await client.get(`key${i}`);
        await client.delete(`key${i}`);
    }
};

beforeEach(async () => {
    client = new RocksDBClient('127.0.0.1', 12345);
    // await client.connect();
});

afterEach(() => {
    client.close();
});

test('should connect to the server', async () => {
    // expect(client.socket).not.toBeNull();
});

test('performance: should measure put operation', async () => {
    const bench = new Bench({ time: 1000 });

    bench
        .add('put', async () => {
            await client.put('key', 'value');
        })
        .add('get', async () => {
            await client.get('key');
        })
        .add('delete', async () => {
            await client.delete('key');
        })
        .todo('unimplemented bench')

    await bench.warmup(); // make results more reliable, ref: https://github.com/tinylibs/tinybench/pull/50
    await bench.run();

    console.table(bench.table());
});

// test('stress test: perform many operations', async () => {
//     const numStressOperations = 1000; // Increase the number for more stress
//     const stressBench = new Bench({ time: 3000 });
//
//     stressBench
//         .add('stress put', async () => {
//             for (let i = 0; i < numStressOperations; i++) {
//                 await client.put(`key${i}`, `value${i}`);
//             }
//         })
//         .add('stress get', async () => {
//             for (let i = 0; i < numStressOperations; i++) {
//                 await client.get(`key${i}`);
//             }
//         })
//         .add('stress delete', async () => {
//             for (let i = 0; i < numStressOperations; i++) {
//                 await client.delete(`key${i}`);
//             }
//         })
//         .todo('unimplemented stress bench');
//
//     await stressBench.warmup();
//     await stressBench.run();
//
//     console.table(stressBench.table());
// });

// test('performance: should measure put operation', async () => {
//     const putOperations = [];
//     const getOperations = [];
//
//     for (let i = 0; i < 10000; i++) {  // 100 здесь - это количество операций, которое вы хотите измерить
//         putOperations.push(client.put(`key${i}`, `value${i}`));
//         getOperations.push(client.get(`key${i}`));
//     }
//
//     console.time('put');
//     await Promise.all(putOperations);
//     console.timeEnd('put');
//
//     console.time('get');
//     await Promise.all(getOperations);
//     console.timeEnd('get');
// });

// test('parallel: should handle 1000 parallel put operations', async () => {
//     const putPromises = [];
//     for (let i = 0; i < numOperations; i++) {
//         putPromises.push(client.put(`key${i}`, `value${i}`));
//     }
//
//     const start = Date.now();
//     await Promise.all(putPromises);
//     const end = Date.now();
//
//     console.log(`${numOperations} parallel put operations took ${end - start} ms`);
// });
//
// test('parallel: should handle 1000 parallel get operations', async () => {
//     const getPromises = [];
//     for (let i = 0; i < numOperations; i++) {
//         getPromises.push(client.get(`key${i}`));
//     }
//
//     const start = Date.now();
//     await Promise.all(getPromises);
//     const end = Date.now();
//
//     console.log(`${numOperations} parallel get operations took ${end - start} ms`);
// });
//
// test('parallel: should handle 1000 parallel delete operations', async () => {
//     const deletePromises = [];
//     for (let i = 0; i < numOperations; i++) {
//         deletePromises.push(client.delete(`key${i}`));
//     }
//
//     const start = Date.now();
//     await Promise.all(deletePromises);
//     const end = Date.now();
//
//     console.log(`${numOperations} parallel delete operations took ${end - start} ms`);
// });
//
// test('parallel: should handle 1000 clients each sending 1000 requests', async () => {
//     const clients = Array.from({ length: numClients }, () => createClient('127.0.0.1', 12345));
//
//     // Directly perform requests without connect
//     const requestPromises = clients.map(client => performClientRequests(client, numRequestsPerClient));
//
//     const start = Date.now();
//     await Promise.all(requestPromises);
//     const end = Date.now();
//
//     console.log(`1000 clients each sending 1000 requests took ${end - start} ms`);
//
//     // Ensure clients are properly closed after requests
//     const closePromises = clients.map(client => client.close());
//     await Promise.all(closePromises);
// });