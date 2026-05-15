use std::collections::HashMap;
/// Performance Metrics Collection Module
///
/// Provides centralized metrics collection for:
/// - Signal read/write operations
/// - Component render performance
/// - Memory usage tracking
/// - Latency histograms
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Central metrics collector
#[derive(Clone, Debug)]
pub struct MetricsCollector {
    /// Signal operation counters by field name
    signal_reads: Arc<Mutex<HashMap<String, AtomicU64>>>,
    signal_writes: Arc<Mutex<HashMap<String, AtomicU64>>>,

    /// Component render timing
    component_renders: Arc<Mutex<HashMap<String, AtomicU64>>>,
    component_render_count: Arc<Mutex<HashMap<String, AtomicU64>>>,

    /// Memory usage estimates (bytes)
    memory_usage: Arc<AtomicU64>,

    /// Global operation counter
    total_operations: Arc<AtomicU64>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            signal_reads: Arc::new(Mutex::new(HashMap::new())),
            signal_writes: Arc::new(Mutex::new(HashMap::new())),
            component_renders: Arc::new(Mutex::new(HashMap::new())),
            component_render_count: Arc::new(Mutex::new(HashMap::new())),
            memory_usage: Arc::new(AtomicU64::new(0)),
            total_operations: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a Signal read operation
    pub fn record_signal_read(&self, field_name: &str, _duration_micros: u64) {
        // Ensure entry exists in HashMap
        let mut reads = self.signal_reads.lock().unwrap_or_else(|e| e.into_inner());
        reads
            .entry(field_name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        self.total_operations.fetch_add(1, Ordering::Relaxed);

        #[cfg(feature = "verbose_metrics")]
        tracing::debug!(
            field = field_name,
            duration_us = _duration_micros,
            "signal_read"
        );
    }

    /// Record a Signal write operation
    pub fn record_signal_write(&self, field_name: &str, _duration_micros: u64) {
        // Ensure entry exists in HashMap
        let mut writes = self.signal_writes.lock().unwrap_or_else(|e| e.into_inner());
        writes
            .entry(field_name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        self.total_operations.fetch_add(1, Ordering::Relaxed);

        #[cfg(feature = "verbose_metrics")]
        tracing::info!(
            field = field_name,
            duration_us = _duration_micros,
            "signal_write"
        );
    }

    /// Record a component render operation
    pub fn record_component_render(&self, component_name: &str, _duration_micros: u64) {
        // Ensure entries exist
        let mut renders = self
            .component_renders
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut counts = self
            .component_render_count
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        renders
            .entry(component_name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(_duration_micros, Ordering::Relaxed);

        counts
            .entry(component_name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        drop(renders);
        drop(counts);

        #[cfg(feature = "verbose_metrics")]
        tracing::info!(
            component = component_name,
            duration_us = _duration_micros,
            "component_render"
        );
    }

    /// Update memory usage estimate
    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage.store(bytes, Ordering::Relaxed);
    }

    /// Get total number of operations
    pub fn total_operations(&self) -> u64 {
        self.total_operations.load(Ordering::Relaxed)
    }

    /// Get current memory usage estimate
    pub fn memory_usage(&self) -> u64 {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Get Signal read count for a field
    pub fn signal_read_count(&self, field_name: &str) -> u64 {
        let reads = self.signal_reads.lock().unwrap_or_else(|e| e.into_inner());
        reads
            .get(field_name)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get Signal write count for a field
    pub fn signal_write_count(&self, field_name: &str) -> u64 {
        let writes = self.signal_writes.lock().unwrap_or_else(|e| e.into_inner());
        writes
            .get(field_name)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get component render count
    pub fn component_render_count(&self, component_name: &str) -> u64 {
        let counts = self
            .component_render_count
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        counts
            .get(component_name)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get component total render time (in microseconds)
    pub fn component_total_render_time(&self, component_name: &str) -> u64 {
        let renders = self
            .component_renders
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        renders
            .get(component_name)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get component average render time (in microseconds)
    pub fn component_avg_render_time(&self, component_name: &str) -> f64 {
        let total = self.component_total_render_time(component_name);
        let count = self.component_render_count(component_name);

        if count == 0 {
            0.0
        } else {
            total as f64 / count as f64
        }
    }

    /// Export metrics as JSON string
    pub fn export_json(&self) -> String {
        #[derive(serde::Serialize)]
        struct MetricsExport {
            total_operations: u64,
            memory_usage_bytes: u64,
            signal_reads: std::collections::HashMap<String, u64>,
            signal_writes: std::collections::HashMap<String, u64>,
            component_renders: Vec<ComponentMetrics>,
        }

        #[derive(serde::Serialize)]
        struct ComponentMetrics {
            name: String,
            render_count: u64,
            total_time_us: u64,
            avg_time_us: f64,
        }

        let mut signal_reads = std::collections::HashMap::new();
        if let Ok(reads) = self.signal_reads.lock() {
            for (key, counter) in reads.iter() {
                signal_reads.insert(key.clone(), counter.load(Ordering::Relaxed));
            }
        }

        let mut signal_writes = std::collections::HashMap::new();
        if let Ok(writes) = self.signal_writes.lock() {
            for (key, counter) in writes.iter() {
                signal_writes.insert(key.clone(), counter.load(Ordering::Relaxed));
            }
        }

        let mut component_renders = Vec::new();
        if let Ok(renders) = self.component_renders.lock() {
            for (name, _) in renders.iter() {
                let count = self.component_render_count(name);
                let total = self.component_total_render_time(name);
                let avg = self.component_avg_render_time(name);

                component_renders.push(ComponentMetrics {
                    name: name.clone(),
                    render_count: count,
                    total_time_us: total,
                    avg_time_us: avg,
                });
            }
        }

        let export = MetricsExport {
            total_operations: self.total_operations.load(Ordering::Relaxed),
            memory_usage_bytes: self.memory_usage.load(Ordering::Relaxed),
            signal_reads,
            signal_writes,
            component_renders,
        };

        serde_json::to_string_pretty(&export).unwrap_or_else(|_| "{}".to_string())
    }

    /// Clear all metrics (for testing)
    pub fn reset(&self) {
        if let Ok(reads) = self.signal_reads.lock() {
            for (_, counter) in reads.iter() {
                counter.store(0, Ordering::Relaxed);
            }
        }
        if let Ok(writes) = self.signal_writes.lock() {
            for (_, counter) in writes.iter() {
                counter.store(0, Ordering::Relaxed);
            }
        }
        if let Ok(renders) = self.component_renders.lock() {
            for (_, counter) in renders.iter() {
                counter.store(0, Ordering::Relaxed);
            }
        }
        if let Ok(counts) = self.component_render_count.lock() {
            for (_, counter) in counts.iter() {
                counter.store(0, Ordering::Relaxed);
            }
        }
        self.total_operations.store(0, Ordering::Relaxed);
        self.memory_usage.store(0, Ordering::Relaxed);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance
static METRICS: once_cell::sync::Lazy<MetricsCollector> =
    once_cell::sync::Lazy::new(MetricsCollector::new);

/// Get the global metrics collector
pub fn global_metrics() -> &'static MetricsCollector {
    &METRICS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = MetricsCollector::new();
        assert_eq!(metrics.total_operations(), 0);
        assert_eq!(metrics.memory_usage(), 0);
    }

    #[test]
    fn test_signal_read_recording() {
        let metrics = MetricsCollector::new();
        metrics.record_signal_read("test_field", 10);
        metrics.record_signal_read("test_field", 15);

        assert_eq!(metrics.signal_read_count("test_field"), 2);
        assert_eq!(metrics.total_operations(), 2);
    }

    #[test]
    fn test_signal_write_recording() {
        let metrics = MetricsCollector::new();
        metrics.record_signal_write("test_field", 20);

        assert_eq!(metrics.signal_write_count("test_field"), 1);
        assert_eq!(metrics.total_operations(), 1);
    }

    #[test]
    fn test_component_render_stats() {
        let metrics = MetricsCollector::new();
        metrics.record_component_render("TestComponent", 1000);
        metrics.record_component_render("TestComponent", 2000);
        metrics.record_component_render("TestComponent", 3000);

        assert_eq!(metrics.component_render_count("TestComponent"), 3);
        assert_eq!(metrics.component_total_render_time("TestComponent"), 6000);
        assert_eq!(metrics.component_avg_render_time("TestComponent"), 2000.0);
    }

    #[test]
    fn test_memory_usage_update() {
        let metrics = MetricsCollector::new();
        metrics.update_memory_usage(1024);
        assert_eq!(metrics.memory_usage(), 1024);
    }

    #[test]
    fn test_reset() {
        let metrics = MetricsCollector::new();
        metrics.record_signal_read("field", 10);
        metrics.record_signal_write("field", 20);
        metrics.update_memory_usage(100);

        assert!(metrics.total_operations() > 0);

        metrics.reset();

        assert_eq!(metrics.total_operations(), 0);
        assert_eq!(metrics.signal_read_count("field"), 0);
        assert_eq!(metrics.signal_write_count("field"), 0);
        assert_eq!(metrics.memory_usage(), 0);
    }
}
