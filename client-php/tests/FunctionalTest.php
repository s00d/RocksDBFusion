<?php
require 'vendor/autoload.php';

use PHPUnit\Framework\TestCase;
use RocksDBClient\RocksDBClient;

class FunctionalTests extends TestCase
{
    private $client;

    protected function setUp(): void
    {
        $this->client = new RocksDBClient('127.0.0.1', 12345);
    }

   public function testPutGet()
   {
       $this->client->put('test_key', 'test_value');
       $value = $this->client->get('test_key');
       $this->assertEquals('test_value', $value);
   }

   public function testDelete()
   {
       $this->client->put('test_key', 'test_value');
       $this->client->delete('test_key');
       try {
           $this->client->get('test_key');
       } catch (\Exception $e) {
           $this->assertEquals('Key not found', $e->getMessage());
       }
   }

   public function testMerge()
   {
       // Передаем JSON-строку как значение в put
       $initial_json = json_encode([
           "employees" => [
               ["first_name" => "john", "last_name" => "doe"],
               ["first_name" => "adam", "last_name" => "smith"]
           ]
       ], JSON_THROW_ON_ERROR);
       $this->client->put('test_key', $initial_json);

       $patch1 = json_encode([
           ["op" => "replace", "path" => "/employees/1/first_name", "value" => "lucy"]
       ], JSON_THROW_ON_ERROR);
       $this->client->merge("test_key", $patch1);

       $patch2 = json_encode([
           ["op" => "replace", "path" => "/employees/0/last_name", "value" => "dow"]
       ], JSON_THROW_ON_ERROR);
       $this->client->merge("test_key", $patch2);

       $val = $this->client->get('test_key');
//        // Получаем значение и декодируем JSON-строку обратно в массив
       $value = json_decode($val, true);

       $expected_value = [
           "employees" => [
               ["first_name" => "john", "last_name" => "dow"],
               ["first_name" => "lucy", "last_name" => "smith"]
           ]
       ];

       // Проверяем, что результат соответствует ожидаемому значению
       $this->assertEquals($expected_value, $value);
   }

}
?>
