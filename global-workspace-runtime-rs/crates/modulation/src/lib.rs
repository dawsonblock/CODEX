pub mod internal_state;
pub mod pressure;
pub mod resonance;
pub mod resources;
pub mod self_model;
pub mod somatic;

pub use internal_state::{update_internal_state, UpdateInputs};
pub use resonance::infer_resonance_tags;
pub use resources::{resource_pressure_from_world, should_conserve, world_resources_critical};
pub use somatic::SomaticMap;
