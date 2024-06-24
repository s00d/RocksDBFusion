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

To enable the metrics server, use the `--metrics` parameter when starting the RocksDB Server. This parameter enables the metrics server, which will run on the same address as the RocksDB Server.

Example:
```sh
rocksdb_server --dbpath ./db_test --address 127.0.0.1:12345 --log-level info --metrics true
```

## Available Metrics

The following metrics are exposed by the server:

- `requests` (Counter): The total number of requests received by the server.
- `request_success_total` (Counter): Total number of successful requests.
- `request_failure_total` (Counter): Total number of failed requests.
- `request_duration_seconds` (Histogram): The duration of the requests in seconds.
- `response_speed_bytes` (Counter): The speed of the response in bytes.
- `cache_hits_total` (Counter): The total number of cache hits.
- `cache_misses_total` (Counter): The total number of cache misses.
- `cache_set_total` (Counter): The total number of cache sets.
- `active_connections` (Gauge): The number of active connections.
- `memory_usage_bytes` (Gauge): Current memory usage of the process in bytes.
- `cpu_usage_percentage` (Gauge): Current CPU usage of the process in percentage.
- `process_uptime_seconds` (Gauge): Uptime of the process in seconds.

## Accessing Metrics

Once the metrics server is enabled, you can access the metrics by navigating to the `/metrics` endpoint of your server in your web browser or using a tool like `curl`.

Example:
```sh
curl http://127.0.0.1:12345/metrics
```

## Prometheus Integration

To integrate the RocksDB Server metrics with Prometheus, add the following job to your Prometheus configuration file:

```yaml
scrape_configs:
  - job_name: 'rocksdb_server'
    static_configs:
      - targets: ['127.0.0.1:12345']
```

Replace `127.0.0.1:12345` with the actual address and port where your RocksDB Server is running.

### Example Prometheus Configuration

Here is an example of a Prometheus configuration file that includes the RocksDB Server metrics:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'rocksdb_server'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['127.0.0.1:12345']
```

Save this configuration to a file, for example, `prometheus.yml`, and start Prometheus with the following command:

```sh
prometheus --config.file=prometheus.yml
```

This configuration will tell Prometheus to scrape the metrics from the RocksDB Server every 15 seconds.

## Grafana Integration

To integrate the RocksDB Server metrics with Grafana, follow these steps:

1. **Add Prometheus Data Source**:
- Open Grafana in your web browser.
- Navigate to `Configuration` -> `Data Sources`.
- Click `Add data source` and select `Prometheus`.
- Set the URL to `http://<PROMETHEUS_HOST>:<PROMETHEUS_PORT>` (e.g., `http://localhost:9090`).
- Click `Save & Test` to verify the connection.

2. **Create a Dashboard**:
- Navigate to `Create` -> `Dashboard`.
- Click `Add new panel`.
- In the `Query` section, select `Prometheus` as the data source.
- Use the following queries to display the various metrics:

### Example Grafana Dashboard Panels

```json
{
  "panels": [
    {
      "title": "Total Requests",
      "type": "stat",
      "targets": [
        {
          "expr": "requests",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Successful Requests",
      "type": "stat",
      "targets": [
        {
          "expr": "request_success_total",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Failed Requests",
      "type": "stat",
      "targets": [
        {
          "expr": "request_failure_total",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Request Duration",
      "type": "graph",
      "targets": [
        {
          "expr": "histogram_quantile(0.95, sum(rate(request_duration_seconds_bucket[5m])) by (le))",
          "legendFormat": "95th percentile"
        },
        {
          "expr": "histogram_quantile(0.50, sum(rate(request_duration_seconds_bucket[5m])) by (le))",
          "legendFormat": "50th percentile"
        }
      ]
    },
    {
      "title": "Response Speed",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(response_speed_bytes[5m])",
          "legendFormat": "Response Speed"
        }
      ]
    },
    {
      "title": "Cache Hits",
      "type": "stat",
      "targets": [
        {
          "expr": "cache_hits_total",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Cache Misses",
      "type": "stat",
      "targets": [
        {
          "expr": "cache_misses_total",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Cache Sets",
      "type": "stat",
      "targets": [
        {
          "expr": "cache_set_total",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Active Connections",
      "type": "stat",
      "targets": [
        {
          "expr": "active_connections",
          "format": "time_series"
        }
      ]
    },
    {
      "title": "Memory Usage",
      "type": "graph",
      "targets": [
        {
          "expr": "memory_usage_bytes",
          "legendFormat": "Memory Usage"
        }
      ]
    },
    {
      "title": "CPU Usage",
      "type": "graph",
      "targets": [
        {
          "expr": "cpu_usage_percentage",
          "legendFormat": "CPU Usage"
        }
      ]
    },
    {
      "title": "Process Uptime",
      "type": "stat",
      "targets": [
        {
          "expr": "process_uptime_seconds",
          "format": "time_series"
        }
      ]
    }
  ]
}
```

This JSON configuration defines a Grafana dashboard with panels for each metric exposed by the RocksDB Server. To use it:

1. Copy the JSON configuration.
2. In Grafana, navigate to `Create` -> `Import`.
3. Paste the JSON into the `Import via panel json` field.
4. Click `Load` and configure the dashboard as needed.
