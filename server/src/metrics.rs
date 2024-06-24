use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use prometheus::{Encoder, TextEncoder, register_histogram, Histogram, register_int_counter, IntCounter, register_int_gauge, IntGauge, Gauge, register_gauge};
use once_cell::sync::Lazy;
use log::{info, error};
use sysinfo::{System, SystemExt, ProcessExt};

pub struct Metrics {
    pub enabled: AtomicBool,
    pub requests: IntCounter,
    pub request_duration: Histogram,
    pub response_speed: IntCounter,
    pub cache_hits: IntCounter,
    pub cache_set: IntCounter,
    pub cache_misses: IntCounter,
    pub active_connections: IntGauge,
    pub memory_usage: Gauge,
    pub cpu_usage: Gauge,
    pub uptime: Gauge,
    pub process_start_time: SystemTime,
    pub request_success: IntCounter,
    pub request_failure: IntCounter,
}

impl Metrics {
    pub fn new(enabled: bool) -> Self {
        let metrics = Self {
            enabled: AtomicBool::new(enabled),
            requests: register_int_counter!(
                "requests",
                "The number of requests"
            ).unwrap(),
            request_success: register_int_counter!(
                "request_success_total",
                "Total number of successful requests"
            ).unwrap(),
            request_failure: register_int_counter!(
                "request_failure_total",
                "Total number of failed requests"
            ).unwrap(),
            request_duration: register_histogram!(
                "request_duration_seconds",
                "The duration of the request in seconds"
            ).unwrap(),
            response_speed: register_int_counter!(
                "response_speed_bytes",
                "The speed of the response in bytes"
            ).unwrap(),
            cache_hits: register_int_counter!(
                "cache_hits_total",
                "The total number of cache hits"
            ).unwrap(),
            cache_set: register_int_counter!(
                "cache_set_total",
                "The total number of cache sets"
            ).unwrap(),
            cache_misses: register_int_counter!(
                "cache_misses_total",
                "The total number of cache misses"
            ).unwrap(),
            active_connections: register_int_gauge!(
                "active_connections",
                "The number of active connections"
            ).unwrap(),
            memory_usage: register_gauge!(
                "memory_usage_bytes",
                "Current memory usage of the process in bytes"
            ).unwrap(),
            cpu_usage: register_gauge!(
                "cpu_usage_percentage",
                "Current CPU usage of the process in percentage"
            ).unwrap(),
            uptime: register_gauge!(
                "process_uptime_seconds",
                "Uptime of the process in seconds"
            ).unwrap(),
            process_start_time: SystemTime::now(),
        };

        metrics
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn gather_metrics() -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();

        // Логируем каждую метрику для отладки
        for family in &metric_families {
            info!("Family: {}", family.get_name());
            for metric in family.get_metric() {
                info!("Metric: {:?}", metric);
            }
        }

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            error!("Failed to encode metrics: {}", e);
        }
        String::from_utf8(buffer).unwrap_or_else(|e| {
            error!("Failed to convert metrics to string: {}", e);
            String::new()
        })
    }

    pub fn update_system_metrics(&self) {
        let mut system = System::new_all();
        system.refresh_all();

        if let Some(process) = system.process(std::process::id() as i32) {
            self.memory_usage.set(process.memory() as f64);
            self.cpu_usage.set(process.cpu_usage() as f64);

            if let Ok(elapsed) = self.process_start_time.elapsed() {
                self.uptime.set(elapsed.as_secs_f64());
            }
        }
    }

    // Метод для инкрементации счетчика запросов
    pub fn inc_requests(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.requests.inc();
        }
    }

    pub fn inc_request_success(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.request_success.inc();
        }
    }

    pub fn inc_request_failure(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.request_failure.inc();
        }
    }

    // Метод для обновления продолжительности запроса
    pub fn observe_request_duration(&self, duration: f64) {
        if self.enabled.load(Ordering::Relaxed) {
            self.request_duration.observe(duration);
        }
    }

    // Аналогичные методы для остальных метрик
    pub fn inc_active_requests(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.active_connections.inc();
        }
    }

    pub fn dec_active_requests(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.active_connections.dec();
        }
    }

    pub fn inc_cache_hits(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.cache_hits.inc();
        }
    }

    pub fn inc_cache_misses(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.cache_misses.inc();
        }
    }

    pub fn inc_cache_set(&self) {
        if self.enabled.load(Ordering::Relaxed) {
            self.cache_set.inc();
        }
    }
}

pub static METRICS: Lazy<Metrics> = Lazy::new(|| Metrics::new(false));

