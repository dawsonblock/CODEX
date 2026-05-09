//! Resource pressure tracking and goal-level resource management.

use runtime_core::types::clamp01;

/// Compute resource pressure from world resources.
pub fn resource_pressure_from_world(world_resources: f64) -> f64 {
    // When resources < 0.35 → strong pressure; linear from 1.0 to 0.0 → 0.0 to 1.0.
    clamp01(1.0 - world_resources)
}

/// Suggests whether to prefer conserve action given pressure.
pub fn should_conserve(resource_pressure: f64) -> bool {
    resource_pressure > 0.65
}

/// More conservative version: also fires at lower threshold.
pub fn world_resources_critical(world_resources: f64) -> bool {
    world_resources < 0.35
}
