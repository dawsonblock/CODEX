//! Global workspace crate: capsule routing and ignition detection.

use runtime_core::InternalState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ─── Capsule ─────────────────────────────────────────────────────────────────

/// A unit of content competing for workspace broadcast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCapsule {
    pub capsule_id: String,
    pub source: String,
    pub content: serde_json::Value,
    /// Combined priority × confidence composite weight.
    pub priority: f64,
    pub confidence: f64,
}

// ─── Router ──────────────────────────────────────────────────────────────────

/// Routes capsules, picking the top-`capacity` by priority × confidence.
pub struct WorkspaceRouter {
    capacity: usize,
}

impl WorkspaceRouter {
    pub fn new(capacity: usize) -> Self {
        WorkspaceRouter {
            capacity: capacity.max(1),
        }
    }

    /// Returns (shortlist, overflow, scores_for_shortlist).
    pub fn route(
        &self,
        capsules: &[WorkspaceCapsule],
        _state: &InternalState,
    ) -> (Vec<WorkspaceCapsule>, Vec<WorkspaceCapsule>, Vec<f64>) {
        let mut indexed: Vec<(f64, usize)> = capsules
            .iter()
            .enumerate()
            .map(|(i, c)| (c.priority * c.confidence, i))
            .collect();
        indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let (sl_idx, ov_idx) = indexed.split_at(self.capacity.min(indexed.len()));
        let shortlist: Vec<WorkspaceCapsule> =
            sl_idx.iter().map(|(_, i)| capsules[*i].clone()).collect();
        let overflow: Vec<WorkspaceCapsule> =
            ov_idx.iter().map(|(_, i)| capsules[*i].clone()).collect();
        let scores: Vec<f64> = sl_idx.iter().map(|(s, _)| *s).collect();
        (shortlist, overflow, scores)
    }

    /// Shannon entropy over normalised scores.
    pub fn selection_entropy(&self, scores: &[f64]) -> f64 {
        let total: f64 = scores.iter().sum();
        if total <= 0.0 {
            return 0.0;
        }
        -scores.iter().filter(|&&s| s > 0.0).fold(0.0, |acc, &s| {
            let p = s / total;
            acc + p * p.ln()
        })
    }
}

// ─── Ignition ─────────────────────────────────────────────────────────────────

/// Detects "global ignition" — whether the workspace broadcast is cohesive enough
/// to drive downstream processing.
pub struct IgnitionDetector {
    threshold: f64,
}

impl IgnitionDetector {
    pub fn new(threshold: f64) -> Self {
        IgnitionDetector { threshold }
    }

    /// Returns true when average dominance exceeds dynamic threshold AND ≥ 2
    /// downstream modules registered significant deltas.
    pub fn check(
        &self,
        shortlist: &[WorkspaceCapsule],
        state: &InternalState,
        downstream_deltas: &std::collections::HashMap<String, f64>,
    ) -> bool {
        if shortlist.is_empty() {
            return false;
        }
        let dominance: f64 = shortlist
            .iter()
            .map(|c| c.priority * c.confidence)
            .sum::<f64>()
            / shortlist.len() as f64;
        let causal_modules = downstream_deltas
            .values()
            .filter(|&&v| v.abs() > 0.05)
            .count();
        let dynamic =
            (self.threshold - 0.2 * state.arousal + 0.15 * state.control).clamp(0.25, 0.95);
        dominance >= dynamic && causal_modules >= 2
    }
}

// ─── Global Workspace ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub cycle_id: u64,
    pub broadcast: WorkspaceBroadcast,
    pub ignition: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBroadcast {
    pub shortlist_ids: Vec<String>,
    pub contents: Vec<serde_json::Value>,
    pub sources: Vec<String>,
    pub selection_entropy: f64,
    pub overflow_count: usize,
}

pub struct GlobalWorkspace {
    router: WorkspaceRouter,
    ignition_detector: IgnitionDetector,
    previous_shortlist_ids: HashSet<String>,
}

impl GlobalWorkspace {
    pub fn new(capacity: usize) -> Self {
        GlobalWorkspace {
            router: WorkspaceRouter::new(capacity),
            ignition_detector: IgnitionDetector::new(0.72),
            previous_shortlist_ids: HashSet::new(),
        }
    }

    pub fn update(
        &mut self,
        cycle_id: u64,
        capsules: &[WorkspaceCapsule],
        state: &InternalState,
        downstream_deltas: std::collections::HashMap<String, f64>,
    ) -> WorkspaceState {
        let (shortlist, overflow, scores) = self.router.route(capsules, state);
        let current_ids: HashSet<String> = shortlist.iter().map(|c| c.capsule_id.clone()).collect();

        let shortlist_delta = if self.previous_shortlist_ids.is_empty() {
            1.0
        } else {
            let union_len = current_ids.union(&self.previous_shortlist_ids).count();
            let inter_len = current_ids
                .intersection(&self.previous_shortlist_ids)
                .count();
            1.0 - inter_len as f64 / union_len.max(1) as f64
        };
        self.previous_shortlist_ids = current_ids;

        let mut dd = downstream_deltas;
        dd.entry("workspace_shortlist".into())
            .or_insert(shortlist_delta);

        let ignition = self.ignition_detector.check(&shortlist, state, &dd);

        let broadcast = WorkspaceBroadcast {
            shortlist_ids: shortlist.iter().map(|c| c.capsule_id.clone()).collect(),
            contents: shortlist.iter().map(|c| c.content.clone()).collect(),
            sources: shortlist.iter().map(|c| c.source.clone()).collect(),
            selection_entropy: self.router.selection_entropy(&scores),
            overflow_count: overflow.len(),
        };

        WorkspaceState {
            cycle_id,
            broadcast,
            ignition,
        }
    }
}
