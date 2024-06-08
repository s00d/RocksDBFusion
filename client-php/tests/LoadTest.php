<?php
require 'vendor/autoload.php';

use PHPUnit\Framework\TestCase;
use RocksDBClient\RocksDBClient;

class LoadTest extends TestCase
{
   private $client;

   protected function setUp(): void
   {
       $this->client = new RocksDBClient('127.0.0.1', 12345);
   }

   public function testBulkPutGet()
   {
       $numOperations = 1000;
       for ($i = 0; $i < $numOperations; $i++) {
           $key = 'key_' . $i;
           $value = 'value_' . $i;
           $this->client->put($key, $value);
       }

       for ($i = 0; $i < $numOperations; $i++) {
           $key = 'key_' . $i;
           $value = $this->client->get($key);
           $this->assertEquals('value_' . $i, $value);
       }
   }

   public function testParallelConnections()
   {
       $numConnections = 10;
       $clients = [];
       for ($i = 0; $i < $numConnections; $i++) {
           $clients[] = new RocksDBClient('127.0.0.1', 12345);
       }

       $numOperations = 100;
       foreach ($clients as $client) {
           for ($i = 0; $i < $numOperations; $i++) {
               $key = 'parallel_key_' . $i;
               $value = 'parallel_value_' . $i;
               $client->put($key, $value);
           }
       }

       foreach ($clients as $client) {
           for ($i = 0; $i < $numOperations; $i++) {
               $key = 'parallel_key_' . $i;
               $value = $client->get($key);
               $this->assertEquals('parallel_value_' . $i, $value);
           }
       }
   }

   public function testPerformance()
   {
       $startTime = microtime(true);

       $numOperations = 100000;
       for ($i = 0; $i < $numOperations; $i++) {
           $key = 'perf_key_' . $i;
           $value = 'perf_value_' . $i;
           $this->client->put($key, $value);
       }

       $endTime = microtime(true);
       $elapsedTime = $endTime - $startTime;
       echo "Time for $numOperations operations: $elapsedTime seconds\n";

       $this->assertLessThan(10, $elapsedTime, "Performance test exceeded 10 seconds");

       // Verifying all added keys
       $startTime = microtime(true);

       for ($i = 0; $i < $numOperations; $i++) {
           $key = 'perf_key_' . $i;
           $expectedValue = 'perf_value_' . $i;
           $actualValue = $this->client->get($key);
           $this->assertEquals($expectedValue, $actualValue);
       }

       $endTime = microtime(true);
       $verificationTime = $endTime - $startTime;

       $this->assertLessThan(10, $verificationTime, "Performance test exceeded 10 seconds");
   }
}
?>
