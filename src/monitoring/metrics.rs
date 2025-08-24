//! Metrics collection and reporting for BWS Web Server
//!
//! This module provides metrics collection, aggregation, and reporting
//! functionality for monitoring server performance and health.

use crate::core::{BwsResult, HealthStatus};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Metrics collector for BWS Web Server
#[derive(Debug)]
pub struct MetricsCollector {
    /// Request counters by status code
    request_counts: Arc<RwLock<HashMap<u16, AtomicU64>>>,

    /// Response time histogram
    response_times: Arc<RwLock<Vec<Duration>>>,

    /// Active connections counter
    active_connections: AtomicU64,

    /// Total bytes served
    bytes_served: AtomicU64,

    /// Error counters by type
    error_counts: Arc<RwLock<HashMap<String, AtomicU64>>>,

    /// Server start time
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            request_counts: Arc::new(RwLock::new(HashMap::new())),
            response_times: Arc::new(RwLock::new(Vec::new())),
            active_connections: AtomicU64::new(0),
            bytes_served: AtomicU64::new(0),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a request with status code and response time
    pub fn record_request(&self, status_code: u16, response_time: Duration, bytes: u64) {
        // Increment request counter
        if let Ok(mut counts) = self.request_counts.write() {
            counts
                .entry(status_code)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        // Record response time (keep only recent samples)
        if let Ok(mut times) = self.response_times.write() {
            times.push(response_time);
            // Keep only last 1000 samples
            if times.len() > 1000 {
                let excess = times.len() - 1000;
                times.drain(0..excess);
            }
        }

        // Add bytes served
        self.bytes_served.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self, error_type: &str) {
        if let Ok(mut errors) = self.error_counts.write() {
            errors
                .entry(error_type.to_string())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> BwsResult<MetricsSnapshot> {
        let request_counts = self
            .request_counts
            .read()
            .map_err(|_| {
                crate::core::BwsError::Internal("Failed to read request counts".to_string())
            })?
            .iter()
            .map(|(k, v)| (*k, v.load(Ordering::Relaxed)))
            .collect();

        let response_times = self
            .response_times
            .read()
            .map_err(|_| {
                crate::core::BwsError::Internal("Failed to read response times".to_string())
            })?
            .clone();

        let error_counts = self
            .error_counts
            .read()
            .map_err(|_| {
                crate::core::BwsError::Internal("Failed to read error counts".to_string())
            })?
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();

        let avg_response_time = if response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = response_times.iter().sum();
            total / response_times.len() as u32
        };

        let p95_response_time = if response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let mut sorted_times = response_times.clone();
            sorted_times.sort();
            let index = (sorted_times.len() as f64 * 0.95) as usize;
            sorted_times.get(index).copied().unwrap_or_default()
        };

        Ok(MetricsSnapshot {
            uptime: self.start_time.elapsed(),
            request_counts,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            bytes_served: self.bytes_served.load(Ordering::Relaxed),
            avg_response_time,
            p95_response_time,
            error_counts,
            health_status: self.get_health_status(),
        })
    }

    /// Get overall health status based on metrics
    fn get_health_status(&self) -> HealthStatus {
        let active_connections = self.active_connections.load(Ordering::Relaxed);
        let error_rate = self.calculate_error_rate();

        if error_rate > 0.1 {
            HealthStatus::Unhealthy
        } else if error_rate > 0.05 || active_connections > 1000 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Calculate current error rate
    fn calculate_error_rate(&self) -> f64 {
        let request_counts = match self.request_counts.read() {
            Ok(counts) => counts,
            Err(_) => return 0.0,
        };

        let total_requests: u64 = request_counts
            .values()
            .map(|v| v.load(Ordering::Relaxed))
            .sum();

        if total_requests == 0 {
            return 0.0;
        }

        let error_requests: u64 = request_counts
            .iter()
            .filter(|(status, _)| **status >= 400)
            .map(|(_, count)| count.load(Ordering::Relaxed))
            .sum();

        error_requests as f64 / total_requests as f64
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of current metrics
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Server uptime
    pub uptime: Duration,

    /// Request counts by status code
    pub request_counts: HashMap<u16, u64>,

    /// Current active connections
    pub active_connections: u64,

    /// Total bytes served
    pub bytes_served: u64,

    /// Average response time
    pub avg_response_time: Duration,

    /// 95th percentile response time
    pub p95_response_time: Duration,

    /// Error counts by type
    pub error_counts: HashMap<String, u64>,

    /// Overall health status
    pub health_status: HealthStatus,
}

impl MetricsSnapshot {
    /// Convert to JSON for API responses
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "uptime_seconds": self.uptime.as_secs(),
            "request_counts": self.request_counts,
            "active_connections": self.active_connections,
            "bytes_served": self.bytes_served,
            "avg_response_time_ms": self.avg_response_time.as_millis(),
            "p95_response_time_ms": self.p95_response_time.as_millis(),
            "error_counts": self.error_counts,
            "health_status": format!("{:?}", self.health_status),
        })
    }
}

/// Global metrics instance
static METRICS: std::sync::OnceLock<MetricsCollector> = std::sync::OnceLock::new();

/// Get global metrics collector
pub fn metrics() -> &'static MetricsCollector {
    METRICS.get_or_init(MetricsCollector::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Record some requests
        collector.record_request(200, Duration::from_millis(100), 1024);
        collector.record_request(404, Duration::from_millis(50), 512);
        collector.record_request(500, Duration::from_millis(200), 256);

        let metrics = collector.get_metrics().unwrap();

        assert_eq!(metrics.request_counts.get(&200), Some(&1));
        assert_eq!(metrics.request_counts.get(&404), Some(&1));
        assert_eq!(metrics.request_counts.get(&500), Some(&1));
        assert_eq!(metrics.bytes_served, 1792);
    }

    #[test]
    fn test_connection_tracking() {
        let collector = MetricsCollector::new();

        collector.increment_connections();
        collector.increment_connections();
        assert_eq!(collector.active_connections.load(Ordering::Relaxed), 2);

        collector.decrement_connections();
        assert_eq!(collector.active_connections.load(Ordering::Relaxed), 1);
    }
}
