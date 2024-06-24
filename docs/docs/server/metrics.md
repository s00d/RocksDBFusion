---
lang: en-US
title: Metrics
sticky: 10
icon: fas fa-chart-line
star: true
dir:
order: 3
category:
- SERVER
---

# Metrics

The RocksDB Server includes a metrics server that exposes various performance and operational metrics for monitoring and analysis. The metrics are exposed in the Prometheus format.

## Enabling Metrics Server

To enable the metrics server, use the `--metrics` parameter when starting the RocksDB Server. This parameter specifies the address and port on which the metrics server will run.

Example:
```sh
rocksdb_server --dbpath ./db_test --address 127.0.0.1:12345 --host 127.0.0.1 --log-level info --metrics 0.0.0.0:9898
```

## Available Metrics

The following metrics are exposed by the server:

- `request_duration_seconds`: The duration of the requests in seconds.
- `response_speed_bytes`: The speed of the response in bytes.
- `cache_hits_total`: The total number of cache hits.
- `cache_misses_total`: The total number of cache misses.
- `active_requests`: The number of active requests.

### Metrics Description

- `request_duration_seconds`
    - **Type**: Histogram
    - **Description**: Measures the duration of requests in seconds.

- `response_speed_bytes`
    - **Type**: Counter
    - **Description**: Tracks the speed of responses in bytes.

- `cache_hits_total`
    - **Type**: Counter
    - **Description**: Counts the total number of cache hits.

- `cache_misses_total`
    - **Type**: Counter
    - **Description**: Counts the total number of cache misses.

- `active_requests`
    - **Type**: Gauge
    - **Description**: Tracks the number of active requests being processed by the server.

## Accessing Metrics

Once the metrics server is enabled, you can access the metrics by navigating to the specified address and port in your web browser or using a tool like `curl`.

Example:
```sh
curl http://127.0.0.1:9898
```

## Prometheus Integration

To integrate the RocksDB Server metrics with Prometheus, add the following job to your Prometheus configuration file:

```yaml
scrape_configs:
  - job_name: 'rocksdb_server'
    static_configs:
      - targets: ['127.0.0.1:9898']
```

Replace `127.0.0.1:9898` with the actual address and port where your metrics server is running.

---

### Command-Line Options for Metrics

- `--metrics <HOST:PORT>`: Enable the metrics server and bind it to the specified host and port.

This page provides an overview of how to enable and access the metrics provided by the RocksDB Server. Monitoring these metrics can help in understanding the server's performance and operational status.

---

With this new page, users will have a clear guide on how to set up and use the metrics server, as well as an understanding of the available metrics and their significance.