//! # Governed Memory: CODEX-Native Bounded Memory Governance
//!
//! This crate provides bounded memory governance primitives that integrate with CODEX
//! evidence, claim, contradiction, and audit systems. It salvages policy concepts from
//! memvid-Human while maintaining CODEX runtime authority.
//!
//! ## Key Principles
//!
//! - **CODEX runtime is authoritative** for action selection (runtime-core unchanged)
//! - **Evidence vault is authoritative** for verified input (evidence crate)
//! - **ClaimStore is authoritative** for the truth window (memory crate)
//! - **Contradiction engine is authoritative** for conflict detection (contradiction crate)
//! - **Governed memory is advisory** — provides decision support, not override
//!
//! ## "Belief" Terminology
//!
//! In this crate, "belief" means a **structured memory assertion record**, not subjective belief
//! or consciousness. It is a data structure that holds collected evidence about a fact, not
//! a claim of subjective experience, sentience, or human-like cognition. See `belief_conflict.rs`
//! for assertion conflict handling.
//!
//! ## No Provider/Network Execution
//!
//! This crate maintains hard invariants:
//! - No API keys stored anywhere
//! - No external service calls (embedding, search, inference)
//! - No .mv2 storage activation
//! - No api_embed activation
//! - Provider output treated as non-authoritative (advisory only)
//!
//! Memory admission, retrieval routing, and conflict handling are local, deterministic,
//! and read-only with respect to the evidence and claim stores.

pub mod admission;
pub mod audit;
pub mod belief_conflict;
pub mod codex_adapter;
pub mod enums;
pub mod policy;
pub mod reason_codes;
pub mod retrieval_intent;
pub mod retrieval_planner;
pub mod schemas;
pub mod source_trust;

// Re-export key types for public API
pub use admission::{MemoryAdmissionDecision, MemoryAdmissionGate};
pub use audit::{GovernedMemoryAuditRecord, MemoryAuditRecord};
pub use belief_conflict::AssertionConflictHandler;
pub use codex_adapter::*;
pub use enums::*;
pub use reason_codes::{ReasonCode, ReasonCodeSource, ReasonSeverity};
pub use retrieval_intent::{RetrievalDecision, RetrievalRouter};
pub use retrieval_planner::{RetrievalPlan, RetrievalPlanner};
pub use schemas::*;
pub use source_trust::{aggregate_confidence, is_source_trusted_for_active_admission, SourceTrust};
