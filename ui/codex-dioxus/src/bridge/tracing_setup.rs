/// Tracing infrastructure for observability and structured logging
/// Provides trace event instrumentation for Signal operations and component renders
use tracing::{info, span, warn, Level};

/// Initialize the tracing subscriber for development
/// Call this once at application startup to enable structured logging
pub fn init_tracing_for_dev() {
    // In production, you would use tracing-subscriber with appropriate formatters
    // For now, this is a placeholder that can be extended with actual logging setup
    info!("Tracing infrastructure initialized");
}

/// Create a span for a Signal read operation
/// Used to track and time read access to Signal fields
#[macro_export]
macro_rules! signal_read_span {
    ($field_name:expr) => {
        tracing::span!(tracing::Level::DEBUG, "signal_read", field = %$field_name)
    };
}

/// Create a span for a Signal write operation
/// Used to track and time write access to Signal fields
#[macro_export]
macro_rules! signal_write_span {
    ($field_name:expr) => {
        tracing::span!(tracing::Level::INFO, "signal_write", field = %$field_name)
    };
}

/// Create a span for a component render operation
/// Used to track render performance and lifecycle
#[macro_export]
macro_rules! component_render_span {
    ($component_name:expr) => {
        tracing::span!(tracing::Level::DEBUG, "component_render", component = %$component_name)
    };
}

/// Log a Signal read access event with duration
/// Helps identify performance hotspots in read-heavy code paths
pub fn trace_signal_read(field_name: &str, duration_micros: u64) {
    if duration_micros > 100 {
        warn!(
            field = %field_name,
            duration_us = duration_micros,
            "Signal read operation exceeded 100μs"
        );
    }
}

/// Log a Signal write access event with duration
/// Helps identify performance issues in write operations
pub fn trace_signal_write(field_name: &str, duration_micros: u64) {
    if duration_micros > 500 {
        warn!(
            field = %field_name,
            duration_us = duration_micros,
            "Signal write operation exceeded 500μs"
        );
    }
}

/// Log a component render event with duration
/// Helps identify slow component renders
pub fn trace_component_render(component_name: &str, duration_micros: u64) {
    if duration_micros > 5000 {
        warn!(
            component = %component_name,
            duration_us = duration_micros,
            "Component render exceeded 5ms"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_completes() {
        // Initialize tracing for testing
        init_tracing_for_dev();
    }

    #[test]
    fn trace_signal_read_completes_without_panic() {
        trace_signal_read("test_field", 50);
        trace_signal_read("test_field", 150); // Should warn when > 100
    }

    #[test]
    fn trace_signal_write_completes_without_panic() {
        trace_signal_write("test_field", 100);
        trace_signal_write("test_field", 600); // Should warn when > 500
    }

    #[test]
    fn trace_component_render_completes_without_panic() {
        trace_component_render("TestComponent", 2000);
        trace_component_render("TestComponent", 6000); // Should warn when > 5ms
    }
}
