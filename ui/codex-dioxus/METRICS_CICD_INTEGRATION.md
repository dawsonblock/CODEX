# Performance Metrics - CI/CD Integration Guide

## Overview

This document explains how performance metrics are collected, stored, and analyzed as part of the CI/CD pipeline.

---

## Metrics Collection During Tests

### Automatic Collection

When tests run, metrics are automatically collected in memory by the `MetricsCollector`:

```rust
// In ui_state.rs
pub fn set_claims(&mut self, claims: Vec<LiveClaimDisplay>) {
    let start = Instant::now();
    // ... operation ...
    global_metrics().record_signal_write("claims", duration_micros);  // AUTO-RECORDED
}
```

**Collected Metrics:**
- ✅ All Signal write operations (13 ops)
- ✅ All Signal read operations (3 ops)
- ✅ All component renders (5 components)
- ✅ Trace events (4 threshold warnings)

### Test Environment

Tests run with the same instrumentation as production:

```bash
cd ui/codex-dioxus
cargo test --bin codex-dioxus --release

# Output: 76 tests passing with metrics collection
# test result: ok. 76 passed; 0 failed; 6 ignored
```

---

## Metrics Export

### During Tests

Current test output includes test count and timing:

```
test result: ok. 76 passed; 0 failed; 6 ignored in 2.34s
```

### For Analysis

Export collected metrics in test code:

```rust
#[test]
fn metrics_baseline() {
    // Run operations...
    
    // Export metrics as JSON
    let metrics_json = global_metrics().export_json();
    
    // Save to file for CI analysis
    std::fs::write("metrics.json", metrics_json)
        .expect("Failed to write metrics");
}
```

### GitHub Actions Artifact Storage

Metrics are archived as CI artifacts (retention: 30 days):

```
Artifacts:
├── ui-performance-metrics/
│   ├── metrics-2024-05-14.json
│   ├── metrics-2024-05-13.json
│   └── metrics-2024-05-12.json
```

---

## Performance Baseline Verification

### Current Baseline (from 76 tests)

```
Signal Operations:
  - Average write duration: 150-500μs
  - Average read duration:  50-150μs

Component Renders:
  - TimelineViewer:         < 15ms average
  - BasisItemsTable:        < 10ms average
  - PressureDynamicsChart:  < 8ms average
  - LongHorizonTraceViewer: < 12ms average
  - ClaimDetailsPanel:      < 20ms average
```

### Regression Detection Thresholds

Performance is considered regressed if:

| Metric | Threshold | Alert Level |
|--------|-----------|-------------|
| Signal write avg | > 600μs | warn |
| Component render avg | > 10ms | warn |
| Any trace warning | > 10/test run | error |
| Test count decrease | < 76 | error |

---

## CI Output Examples

### Success Case (No Regressions)

```
name: CI
on: [push, pull_request]

ui-tests:
  ✅ Run UI tests (with performance metrics)
  ✅ Report test results  
  ✅ Archive performance metrics
  ✅ Performance baseline check (UI)
  
Result: All 76 tests passed ✅
```

### Regression Detection

```
❌ Performance regression detected:
   TimelineViewer: 15000μs avg (was 10000μs)
   Regression: +50% ❌
   
Action: Fail CI, block merge
```

---

## Analyzing Metrics

### Manual Query: Find Slow Operations

```bash
# Export metrics from CI artifact
cd ui/codex-dioxus
cargo test --bin codex-dioxus -- --nocapture 2>&1 | grep -A5 "slow\|WARN"
```

### Automated Analysis Script

Create `analyze_metrics.rs`:

```rust
fn main() {
    let metrics_json = std::fs::read_to_string("metrics.json")
        .expect("metrics.json not found");
    
    let metrics: Metrics = serde_json::from_str(&metrics_json)
        .expect("Invalid JSON");
    
    // Find slow operations
    for (field, stats) in &metrics.signal_writes {
        if stats.avg_micros > 600 {
            println!("⚠️ SLOW WRITE: {} = {}μs", field, stats.avg_micros);
        }
    }
    
    // Find slow renders
    for (component, stats) in &metrics.component_renders {
        if stats.avg_micros > 10000 {
            println!("⚠️ SLOW RENDER: {} = {}μs", component, stats.avg_micros);
        }
    }
}
```

### Download Artifacts for Historical Analysis

```bash
# Download metrics from GitHub Actions
gh run download <run_id> --name ui-performance-metrics

# Analyze trends
python3 analyze_metrics_trend.py metrics-*.json
```

---

## Metrics Interpretation

### Read the JSON Export

```json
{
  "signal_writes": {
    "claims": {
      "count": 42,
      "total_micros": 18900,
      "avg_micros": 450
    }
  },
  "signal_reads": {
    "claims_by_id": {
      "count": 280,
      "total_micros": 22400,
      "avg_micros": 80
    }
  },
  "component_renders": {
    "TimelineViewer": {
      "count": 12,
      "total_micros": 156000,
      "avg_micros": 13000
    }
  }
}
```

**Interpretation:**
- `count`: Number of times operation/render occurred
- `total_micros`: Sum of all durations
- `avg_micros`: Mean duration (total / count)

### Common Issues

**Issue: Signal write avg = 2000μs (should be <600μs)**

```
Diagnosis:
  1. Check count - if only 1-2 operations, might be one-time cost
  2. Check if claim collection was large
  3. Profile with flamegraph to find bottleneck
  
Fix:
  - Paginate large collections
  - Parallelize indexing
  - Consider batching updates
```

**Issue: Component render avg = 25ms (should be <10ms)**

```
Diagnosis:
  1. Check if multiple renders in one test
  2. Profile Dioxus VirtualDOM diffing
  3. Check for unnecessary re-renders
  
Fix:
  - Use virtual scrolling
  - Memoize expensive selectors
  - Extract to separate components
```

---

## Continuous Performance Monitoring

### Trend Tracking

Keep historical metrics to spot trends:

```
Week 1: avg_render = 8ms  ✅
Week 2: avg_render = 9ms  ✅  
Week 3: avg_render = 11ms ⚠️ (gradual regression)
Week 4: avg_render = 15ms ❌ (critical - needs investigation)
```

### Performance Budget

Define acceptable performance budget per operation:

```yaml
performance_budget:
  signal_writes:
    claims: 500μs        # Allow up to 500μs
    evidence: 500μs
    pressure: 600μs
    cycle: 100μs
  
  component_renders:
    TimelineViewer: 15ms
    BasisItemsTable: 10ms
    PressureDynamicsChart: 8ms
```

---

## Metrics Reset Between Test Runs

The `MetricsCollector` maintains global state. Between test runs:

```rust
#[after_test]
fn cleanup() {
    global_metrics().reset();  // Clear for next run
}
```

This ensures metrics from one test don't affect the next.

---

## Future Enhancements

### Phase 14+: Advanced Metrics

- [ ] Histogram buckets (p50, p95, p99)
- [ ] Per-cycle segmentation
- [ ] Memory allocation tracking
- [ ] Garbage collection impact
- [ ] Browser DevTools integration
- [ ] Real User Monitoring (RUM)

### Planned Integrations

- [ ] Datadog APM integration
- [ ] CloudWatch custom metrics
- [ ] Prometheus format export
- [ ] Grafana dashboard templates
- [ ] Slack notifications for regressions

---

## Summary

✅ Metrics are automatically collected during all tests  
✅ Performance baselines are defined for all operations  
✅ CI pipeline verifies baselines and blocks regressions  
✅ Artifacts are stored for historical trend analysis  
✅ Structured logging enables detailed performance investigation  

The infrastructure is ready for team-wide monitoring and performance awareness.
