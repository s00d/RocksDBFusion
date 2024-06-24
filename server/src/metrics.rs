use prometheus::{Encoder, TextEncoder, register_histogram, Histogram, register_int_counter, IntCounter, register_int_gauge, IntGauge};
use once_cell::sync::Lazy;
use log::{info, error};

pub struct Metrics {
    pub enabled: bool,
    pub requests: IntCounter,
    pub request_duration: Histogram,
    pub response_speed: IntCounter,
    pub cache_hits: IntCounter,
    pub cache_set: IntCounter,
    pub cache_misses: IntCounter,
    pub active_requests: IntGauge,
}

impl Metrics {
    pub fn new(enabled: bool) -> Self {
        let metrics = Self {
            enabled,
            requests: register_int_counter!(
                "requests",
                "The number of requests"
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
            active_requests: register_int_gauge!(
                "active_requests",
                "The number of active requests"
            ).unwrap(),
        };

        metrics
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

    // Метод для инкрементации счетчика запросов
    pub fn inc_requests(&self) {
        if self.enabled {
            self.requests.inc();
        }
    }

    // Метод для обновления продолжительности запроса
    pub fn observe_request_duration(&self, duration: f64) {
        if self.enabled {
            self.request_duration.observe(duration);
        }
    }

    // Аналогичные методы для остальных метрик
    pub fn inc_active_requests(&self) {
        if self.enabled {
            self.active_requests.inc();
        }
    }

    pub fn dec_active_requests(&self) {
        if self.enabled {
            self.active_requests.dec();
        }
    }

    pub fn inc_cache_hits(&self) {
        if self.enabled {
            self.cache_hits.inc();
        }
    }

    pub fn inc_cache_misses(&self) {
        if self.enabled {
            self.cache_misses.inc();
        }
    }

    pub fn inc_cache_set(&self) {
        if self.enabled {
            self.cache_set.inc();
        }
    }
}

pub static METRICS: Lazy<Metrics> = Lazy::new(|| Metrics::new(false)); // По умолчанию метрики выключены

