# Performance Observability Guide

## Overview

This guide explains how the CODEX UI framework monitors and tracks performance using Signal operation metrics and component render timing. The infrastructure is built on three layers:

1. **Metrics Collection** (`src/bridge/metrics.rs`) - Thread-safe global metrics aggregation
2. **Signal Instrumentation** (`src/bridge/ui_state.rs`) - Timing for reactive state operations
3. **Component Instrumentation** (`src/bridge/instrumentation.rs`) - Render performance tracking
4. **Structured Logging** (`src/bridge/tracing_setup.rs`) - Threshold-based trace events

---

## Architecture

### Metrics Collection Layer

The `MetricsCollector` provides a thread-safe singleton for aggregating performance data:

```rust
use crate::bridge::metrics::global_metrics;

// Record Signal write operation
global_metrics().record_signal_write("field_name", duration_micros);

// Record Signal read operation
global_metrics().record_signal_read("field_name", duration_micros);

// Record component render
global_metrics().record_component_render("ComponentName", duration_micros);

// Export metrics as JSON
let json = global_metrics().export_json();
```

**Key Features:**
- Per-field tracking for all Signal operations
- Per-component tracking for render performance
- Thread-safe: uses `Arc<Mutex<HashMap<String, AtomicU64>>>`
- Memory usage tracking via `get_memory_usage()`
- JSON export for external analysis

### Signal Instrumentation

Critical write operations in `UIRuntimeState` are instrumented with timing:

| Operation | Threshold | Purpose |
|-----------|-----------|---------|
| `set_claims` | 500μs | Track claim collection updates |
| `set_evidence` | 500μs | Track evidence collection updates |
| `add_pressure_metric` | 500μs | Monitor pressure state changes |
| `set_current_cycle` | 100μs | Fast cycle transitions |
| `set_loading` | 50μs | Quick loading flag updates |

**Instrumentation Pattern:**
```rust
pub fn set_claims(&mut self, claims: Vec<LiveClaimDisplay>) {
    let start = Instant::now();
    info!("Signal: updating claims collection");  // trace event
    
    // ... perform operation ...
    
    let duration = start.elapsed().as_micros() as u64;
    global_metrics().record_signal_write("claims", duration);  // metrics
    trace_signal_write("claims", duration);  // conditional trace
}
```

### Component Render Instrumentation

All 5 UI components measure render performance:

```
TimelineViewer              - Timeline rendering duration
BasisItemsTable             - Table with conditional logic
PressureDynamicsChart       - Chart visualization timing
LongHorizonTraceViewer      - Cycle selector rendering
ClaimDetailsPanel           - Complex conditional rendering
```

**Instrumentation Pattern:**
```rust
pub fn ComponentName() -> Element {
    let timer = start_component_render_timer();
    
    // ... component setup and logic ...
    
    let element = rsx! { /* ... */ };
    end_component_render_timer("ComponentName", timer);  // records + traces
    element
}
```

### Structured Logging with Tracing

The `tracing` crate provides structured logging with span support:

**Trace Events Emitted:**
- Signal operations: `info!()` events logged at operation start
- Performance warnings: `warn!()` events when operations exceed thresholds

**Threshold Warnings:**
```rust
// Signal reads warning threshold: > 100μs
trace_signal_read("field_name", 150);  // emits warn!

// Signal writes warning threshold: > 500μs
trace_signal_write("field_name", 600);  // emits warn!

// Component renders warning threshold: > 5ms
trace_component_render("ComponentName", 6000);  // emits warn!
```

---

## Metrics Analysis

### Exporting Metrics

Export collected metrics to JSON format for analysis:

```rust
use crate::bridge::metrics::global_metrics;

let metrics_json = global_metrics().export_json();
println!("{}", metrics_json);
```

**Example Output:**
```json
{
  "signal_writes": {
    "claims": {"count": 42, "total_micros": 18500, "avg_micros": 440},
    "evidence": {"count": 38, "total_micros": 16200, "avg_micros": 426},
    "current_cycle": {"count": 150, "total_micros": 8400, "avg_micros": 56}
  },
  "signal_reads": {
    "claims_by_id": {"count": 280, "total_micros": 22400, "avg_micros": 80}
  },
  "component_renders": {
    "TimelineViewer": {"count": 12, "total_micros": 156000, "avg_micros": 13000},
    "BasisItemsTable": {"count": 18, "total_micros": 94500, "avg_micros": 5250}
  }
}
```

### Performance Queries

**Slow Signal Operations:**
```rust
// Find all Signal writes that exceed 500μs average
let metrics = global_metrics().export_json();
for (field, data) in metrics.signal_writes {
    if data.avg_micros > 500 {
        println!("SLOW: {} averaged {}μs", field, data.avg_micros);
    }
}
```

**Slow Components:**
```rust
// Find all components that exceed 5ms average render time
for (component, data) in metrics.component_renders {
    if data.avg_micros > 5000 {
        println!("SLOW RENDER: {} = {}μs", component, data.avg_micros);
    }
}
```

**Memory Usage:**
```rust
let mem_usage = global_metrics().get_memory_usage();
println!("Metrics memory: {} bytes", mem_usage);
```

---

## CI/CD Integration

### Collecting Metrics in Tests

Metrics are automatically collected during all tests. To access metrics in test code:

```rust
#[test]
fn verify_performance_baseline() {
    // Run test code...
    let metrics = global_metrics().export_json();
    
    // Assert performance baseline
    assert!(
        metrics.component_renders["TimelineViewer"].avg_micros < 15000,
        "Timeline render regressed above 15ms baseline"
    );
}
```

### Performance Regression Detection

Add this to CI pipeline to detect performance regressions:

```bash
# Run tests and collect metrics
cargo test --release

# Export and analyze metrics
cargo run --bin codex-metrics-analyzer < metrics.json

# Fail if regressions detected above threshold (5% slower)
if [ $regression_percent -gt 5 ]; then
    echo "Performance regression detected: ${regression_percent}% slower"
    exit 1
fi
```

### GitHub Actions Integration

Add to `.github/workflows/ci.yml`:

```yaml
- name: Run performance tests
  run: cargo test --release -- --nocapture
  
- name: Export metrics
  run: cargo run --release --bin codex-metrics-analyzer > metrics.json
  
- name: Check performance baseline
  run: |
    if grep -q '"TimelineViewer".*"avg_micros": [0-9]\{5,\}' metrics.json; then
      echo "⚠️ Performance regression: TimelineViewer render time increased"
      exit 1
    fi
```

---

## Monitoring in Production

### Log Aggregation

Configure your favorite log aggregation service (ELK, Datadog, etc.) to ingest tracing output:

```bash
# Configure RUST_LOG environment variable
export RUST_LOG=codex_dioxus::bridge=debug

# Enable tracing subscriber in application
init_tracing_for_dev();
```

### Alerts

Set up alerts for performance thresholds:

| Alert | Threshold | Action |
|-------|-----------|--------|
| Slow Signal Write | > 1000μs | Page on-call |
| Slow Component Render | > 10ms | Create performance ticket |
| High Error Rate | > 1% | Page on-call |

---

## Best Practices

### 1. Interpret Averages with Caution

Always check percentile distributions, not just averages:

```rust
// BAD: Only looking at average
avg_render = 5000;  // Could hide 1ms renders and 50ms renders

// GOOD: Look at bucketed distribution
p50: 2000μs  (typical)
p95: 8000μs  (occasionally slow)
p99: 12000μs (rare outliers)
```

### 2. Control Variables

Test same scenario repeatedly to reduce variance:

```rust
// Multiple iterations reduce noise
for _ in 0..100 {
    render_component();
}
let metrics = global_metrics().export_json();
```

### 3. Profile Under Load

Collect metrics with realistic load patterns:

```rust
// Simulate concurrent users
tokio::spawn(async { update_claims().await });
tokio::spawn(async { update_evidence().await });
tokio::spawn(async { update_pressure().await });

// Let system stabilize
tokio::time::sleep(Duration::from_secs(5)).await;

// Now collect metrics
let metrics = global_metrics().export_json();
```

### 4. Track Over Time

Store metrics in a time-series database to identify trends:

```
TimelineViewer avg render time:
Day 1:  2500μs  (baseline)
Day 2:  2600μs  (+4%)
Day 3:  2900μs  (+16% - regression alert!)
```

---

## Debugging Performance Issues

### Common Slow Patterns

**Pattern: Slow Signal Writes**
```
Problem: set_claims() regularly exceeds 500μs
Causes:
  - Large claim collections (O(n) indexing)
  - Blocking I/O in Signal setter
  - Lock contention on metrics collection

Solution:
  - Profile with flamegraph
  - Consider pagination for large collections
  - Use separate thread for I/O
```

**Pattern: Slow Component Renders**
```
Problem: TimelineViewer renders > 15ms
Causes:
  - Large list rendering (O(n²) diff)
  - Complex conditional logic in rsx!
  - Frequent re-renders from parent

Solution:
  - Use virtual scrolling
  - Memoize expensive computations
  - Profile with React DevTools
```

### Profiling Tools

1. **Flamegraph**: Show where time is spent
   ```bash
   cargo flamegraph --bin codex-dioxus
   ```

2. **Criterion.rs**: Statistical performance testing
   ```bash
   cargo bench
   ```

3. **perf**: System-level profiling
   ```bash
   perf record --call-graph=dwarf ./target/release/codex-dioxus
   ```

---

## Summary

The CODEX observability stack provides:

✅ **Automatic collection** of Signal operation metrics  
✅ **Component render** performance tracking  
✅ **Threshold-based** structured logging  
✅ **JSON export** for external analysis  
✅ **Test integration** for CI/CD pipelines  
✅ **Tested observability** architecture  

This enables rapid identification and resolution of performance regressions before reaching production.
