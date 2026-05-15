/// Component instrumentation utilities for performance tracking
///
/// Provides macros and helpers to track component render performance
use crate::bridge::metrics::global_metrics;
use crate::bridge::tracing_setup::trace_component_render;
use std::time::Instant;
use tracing::debug;

/// Start a component render timer
/// Returns an Instant that can be passed to end_component_render_timer
#[inline]
pub fn start_component_render_timer() -> Instant {
    Instant::now()
}

/// End a component render timer and record metrics
///
/// # Example
/// ```
/// let timer = start_component_render_timer();
/// // ... component rendering ...
/// end_component_render_timer("MyComponent", timer);
/// ```
#[inline]
pub fn end_component_render_timer(component_name: &str, start: Instant) {
    let duration_micros = start.elapsed().as_micros() as u64;
    debug!(
        "Component '{}' rendered in {}μs",
        component_name, duration_micros
    );
    global_metrics().record_component_render(component_name, duration_micros);
    trace_component_render(component_name, duration_micros);
}

/// Record a single component render without explicit timer management
/// Useful for quick measurements outside of the rendering function
#[inline]
pub fn record_render_time(component_name: &str, duration_micros: u64) {
    debug!(
        "Component '{}' render time recorded: {}μs",
        component_name, duration_micros
    );
    global_metrics().record_component_render(component_name, duration_micros);
    trace_component_render(component_name, duration_micros);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timer_basic() {
        let timer = start_component_render_timer();
        thread::sleep(Duration::from_micros(100));
        let elapsed = timer.elapsed().as_micros() as u64;
        assert!(elapsed >= 100);
    }

    #[test]
    fn test_record_render_time() {
        let _metrics = crate::bridge::metrics::MetricsCollector::new();
        // This just verifies the function compiles and runs
        record_render_time("TestComponent", 1000);
        // Note: Can't easily verify in tests without access to collected metrics
    }
}
