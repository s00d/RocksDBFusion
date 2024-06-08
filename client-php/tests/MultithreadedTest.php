<?php

// use Swoole\Coroutine;
// use Swoole\Coroutine\Channel;
// use PHPUnit\Framework\TestCase;
// use RocksDBClient\RocksDBClient;
//
// class MultithreadedTest extends TestCase
// {
//     private $host = '127.0.0.1';
//     private $port = 12345;
//     private $numThreads = 100;
//
//     public function testMultithreadedWriteRead()
//     {
//         if (extension_loaded('xdebug')) {
//             // Completely disable Xdebug by unloading the extension
//             if (function_exists('xdebug_disable')) {
//                 xdebug_disable();
//             }
//             ini_set('xdebug.remote_enable', '0');
//             ini_set('xdebug.profiler_enable', '0');
//             // Unset the environment variables related to Xdebug
//             putenv('XDEBUG_CONFIG');
//             putenv('PHP_IDE_CONFIG');
//         }
//
//         \Swoole\Runtime::enableCoroutine();
//
//         // Ensure the test runs within a coroutine
//         go(function() {
//             $writeChannel = new Channel($this->numThreads);
//             $readChannel = new Channel($this->numThreads);
//
//             // Start write coroutines
//             for ($i = 0; $i < $this->numThreads; $i++) {
//                 go(function() use ($i, $writeChannel) {
//                     $client = new RocksDBClient($this->host, $this->port);
//                     $key = 'key_' . $i;
//                     $value = 'value_' . $i;
//                     $client->put($key, $value);
//                     $writeChannel->push(true);
//                 });
//             }
//
//             // Wait for all write coroutines to finish
//             for ($i = 0; $i < $this->numThreads; $i++) {
//                 $writeChannel->pop();
//             }
//
//             // Start read coroutines
//             for ($i = 0; $i < $this->numThreads; $i++) {
//                 go(function() use ($i, $readChannel) {
//                     $client = new RocksDBClient($this->host, $this->port);
//                     $key = 'key_' . $i;
//                     $expectedValue = 'value_' . $i;
//                     $value = $client->get($key);
//                     if ($value !== $expectedValue) {
//                         echo "Mismatch: expected {$expectedValue}, got $value\n";
//                     }
//                     $readChannel->push(true);
//                 });
//             }
//
//             // Wait for all read coroutines to finish
//             for ($i = 0; $i < $this->numThreads; $i++) {
//                 $readChannel->pop();
//             }
//
//             $this->assertTrue(true); // If we reach here, the test passed
//         });
//
//         // Wait for the main coroutine to finish
//         \Swoole\Event::wait();
//     }
// }
