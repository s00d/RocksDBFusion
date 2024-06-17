package rocksdbclient_test

import (
	"encoding/json"
	rocksdbclient "github.com/s00d/RocksDBFusion/rocksdb-client-go/src"
	"reflect"
	"testing"
	"time"
)

func stringPtr(s string) *string {
	return &s
}

func TestPutGet(t *testing.T) {
	client := rocksdbclient.NewRocksDBClient("127.0.0.1", 12345, nil, 10*time.Second, 2*time.Second)
	defer client.Close()

	if err := client.Connect(); err != nil {
		t.Fatalf("failed to connect: %v", err)
	}

	_, err := client.Put(stringPtr("test_key"), stringPtr("test_value"), nil, nil)
	if err != nil {
		t.Fatalf("failed to put value: %v", err)
	}

	response, err := client.Get(stringPtr("test_key"), nil, nil, nil)
	if err != nil {
		t.Fatalf("failed to get value: %v", err)
	}

	if response.Result != "test_value" {
		t.Fatalf("expected 'test_value', got '%s'", response.Result)
	}
}

func TestDelete(t *testing.T) {
	client := rocksdbclient.NewRocksDBClient("127.0.0.1", 12345, nil, 10*time.Second, 2*time.Second)
	defer client.Close()

	if err := client.Connect(); err != nil {
		t.Fatalf("failed to connect: %v", err)
	}

	_, err := client.Put(stringPtr("test_key"), stringPtr("test_value"), nil, nil)
	if err != nil {
		t.Fatalf("failed to put value: %v", err)
	}

	_, err = client.Delete(stringPtr("test_key"), nil, nil)
	if err != nil {
		t.Fatalf("failed to delete value: %v", err)
	}

	response, err := client.Get(stringPtr("test_key"), nil, stringPtr("default_value"), nil)
	if err != nil {
		t.Fatalf("failed to get value: %v", err)
	}

	if response.Result != "default_value" {
		t.Fatalf("expected 'default_value', got '%s'", response.Result)
	}
}

func TestMerge(t *testing.T) {
	client := rocksdbclient.NewRocksDBClient("127.0.0.1", 12345, nil, 10*time.Second, 2*time.Second)
	defer client.Close()

	if err := client.Connect(); err != nil {
		t.Fatalf("failed to connect: %v", err)
	}

	initialJson := `{"employees":[{"first_name":"john","last_name":"doe"},{"first_name":"adam","last_name":"smith"}]}`
	_, err := client.Put(stringPtr("test_key"), stringPtr(initialJson), nil, nil)
	if err != nil {
		t.Fatalf("failed to put initial value: %v", err)
	}

	patch1 := `[{"op":"replace","path":"/employees/1/first_name","value":"lucy"}]`
	_, err = client.Merge(stringPtr("test_key"), stringPtr(patch1), nil, nil)
	if err != nil {
		t.Fatalf("failed to merge patch1: %v", err)
	}

	patch2 := `[{"op":"replace","path":"/employees/0/last_name","value":"dow"}]`
	_, err = client.Merge(stringPtr("test_key"), stringPtr(patch2), nil, nil)
	if err != nil {
		t.Fatalf("failed to merge patch2: %v", err)
	}

	response, err := client.Get(stringPtr("test_key"), nil, nil, nil)
	if err != nil {
		t.Fatalf("failed to get value: %v", err)
	}

	expectedValue := `{"employees":[{"first_name":"john","last_name":"dow"},{"first_name":"lucy","last_name":"smith"}]}`
	var result map[string]interface{}
	var expected map[string]interface{}
	if err := json.Unmarshal([]byte(response.Result), &result); err != nil {
		t.Fatalf("failed to unmarshal result: %v", err)
	}
	if err := json.Unmarshal([]byte(expectedValue), &expected); err != nil {
		t.Fatalf("failed to unmarshal expected value: %v", err)
	}

	if !reflect.DeepEqual(result, expected) {
		t.Fatalf("expected '%v', got '%v'", expected, result)
	}
}
