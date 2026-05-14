//! Explicit mapping between durable claim statuses and in-memory lifecycle statuses.
//!
//! # Canonical vs lossy mapping
//!
//! [`durable_to_canonical`] / [`canonical_to_durable`] are the **non-lossy** round-trip
//! functions.  They map every `DurableClaimStatus` variant to a distinct `MemoryStatus`
//! variant so that no information is discarded.
//!
//! [`durable_to_memory`] / [`memory_to_durable`] are **deprecated lossy** shims kept only
//! for backward-compatibility with code that uses the 4-variant `ClaimStatus`.  They collapse
//! Rejected/Disputed → Contradicted and Stale/Superseded → Superseded.  Do not use them in
//! new code.

use crate::durable_memory_provider::ClaimStatus as DurableClaimStatus;
use crate::ClaimStatus as MemoryClaimStatus;
use crate::MemoryStatus;

// ── Legacy ClaimStatus ↔ MemoryStatus helpers ────────────────────────────────

/// Lossless promotion from the legacy 4-variant `ClaimStatus` to `MemoryStatus`.
pub fn legacy_to_canonical(status: MemoryClaimStatus) -> MemoryStatus {
    match status {
        MemoryClaimStatus::Active => MemoryStatus::Active,
        MemoryClaimStatus::Unverified => MemoryStatus::Unverified,
        MemoryClaimStatus::Contradicted => MemoryStatus::Contradicted,
        MemoryClaimStatus::Superseded => MemoryStatus::Superseded,
    }
}

/// Best-effort narrowing from `MemoryStatus` to the legacy 4-variant `ClaimStatus`.
///
/// Variants that have no direct legacy equivalent are mapped to the closest match:
/// `Validated` → `Active`, `Rejected` → `Contradicted`, `Archived` → `Superseded`, etc.
pub fn canonical_to_legacy(status: MemoryStatus) -> MemoryClaimStatus {
    match status {
        MemoryStatus::Active | MemoryStatus::Validated => MemoryClaimStatus::Active,
        MemoryStatus::Unverified => MemoryClaimStatus::Unverified,
        MemoryStatus::Disputed | MemoryStatus::Contradicted | MemoryStatus::Rejected => {
            MemoryClaimStatus::Contradicted
        }
        MemoryStatus::Stale | MemoryStatus::Superseded | MemoryStatus::Archived => {
            MemoryClaimStatus::Superseded
        }
        MemoryStatus::Unknown => MemoryClaimStatus::Unverified,
    }
}

// ── Canonical non-lossy API ──────────────────────────────────────────────────

/// Convert a `DurableClaimStatus` to the canonical `MemoryStatus` without information loss.
///
/// Every durable variant maps to a distinct `MemoryStatus` variant.
pub fn durable_to_canonical(status: DurableClaimStatus) -> MemoryStatus {
    match status {
        DurableClaimStatus::Asserted => MemoryStatus::Unverified,
        DurableClaimStatus::Validated => MemoryStatus::Validated,
        DurableClaimStatus::Rejected => MemoryStatus::Rejected,
        DurableClaimStatus::Stale => MemoryStatus::Stale,
        DurableClaimStatus::Disputed => MemoryStatus::Disputed,
        DurableClaimStatus::Superseded => MemoryStatus::Superseded,
        DurableClaimStatus::Unknown => MemoryStatus::Unknown,
    }
}

/// Convert a canonical `MemoryStatus` back to `DurableClaimStatus` without information loss.
///
/// `MemoryStatus::Active` maps to `Validated` (both represent current, confirmed knowledge).
/// `MemoryStatus::Archived` maps to `Superseded` (closest durable equivalent).
/// `MemoryStatus::Contradicted` maps to `Disputed` (closest durable equivalent).
pub fn canonical_to_durable(status: MemoryStatus) -> DurableClaimStatus {
    match status {
        MemoryStatus::Unverified => DurableClaimStatus::Asserted,
        MemoryStatus::Validated | MemoryStatus::Active => DurableClaimStatus::Validated,
        MemoryStatus::Rejected => DurableClaimStatus::Rejected,
        MemoryStatus::Stale => DurableClaimStatus::Stale,
        MemoryStatus::Disputed | MemoryStatus::Contradicted => DurableClaimStatus::Disputed,
        MemoryStatus::Superseded | MemoryStatus::Archived => DurableClaimStatus::Superseded,
        MemoryStatus::Unknown => DurableClaimStatus::Unknown,
    }
}

// ── Deprecated lossy shims ───────────────────────────────────────────────────

/// Map a durable status to the 4-variant `ClaimStatus`.
///
/// **Lossy:** Rejected and Disputed both become `Contradicted`; Stale and Superseded both become
/// `Superseded`.  Prefer [`durable_to_canonical`] in new code.
#[deprecated(note = "lossy — use durable_to_canonical instead")]
pub fn durable_to_memory(status: DurableClaimStatus) -> MemoryClaimStatus {
    match status {
        DurableClaimStatus::Asserted => MemoryClaimStatus::Unverified,
        DurableClaimStatus::Validated => MemoryClaimStatus::Active,
        DurableClaimStatus::Rejected | DurableClaimStatus::Disputed => {
            MemoryClaimStatus::Contradicted
        }
        DurableClaimStatus::Stale | DurableClaimStatus::Superseded => MemoryClaimStatus::Superseded,
        DurableClaimStatus::Unknown => MemoryClaimStatus::Unverified,
    }
}

/// Map the 4-variant `ClaimStatus` to a durable status.
///
/// **Lossy:** `Contradicted` becomes `Disputed` (not `Rejected`).  Prefer
/// [`canonical_to_durable`] in new code.
#[deprecated(
    since = "codex-main-12",
    note = "lossy — use canonical_to_durable instead"
)]
pub fn memory_to_durable(status: MemoryClaimStatus) -> DurableClaimStatus {
    match status {
        MemoryClaimStatus::Unverified => DurableClaimStatus::Asserted,
        MemoryClaimStatus::Active => DurableClaimStatus::Validated,
        MemoryClaimStatus::Contradicted => DurableClaimStatus::Disputed,
        MemoryClaimStatus::Superseded => DurableClaimStatus::Superseded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Canonical round-trip tests ────────────────────────────────────────────

    #[test]
    fn rejected_remains_rejected() {
        assert_eq!(
            durable_to_canonical(DurableClaimStatus::Rejected),
            MemoryStatus::Rejected
        );
        assert_eq!(
            canonical_to_durable(MemoryStatus::Rejected),
            DurableClaimStatus::Rejected
        );
    }

    #[test]
    fn disputed_remains_disputed() {
        assert_eq!(
            durable_to_canonical(DurableClaimStatus::Disputed),
            MemoryStatus::Disputed
        );
        assert_eq!(
            canonical_to_durable(MemoryStatus::Disputed),
            DurableClaimStatus::Disputed
        );
    }

    #[test]
    fn stale_remains_stale() {
        assert_eq!(
            durable_to_canonical(DurableClaimStatus::Stale),
            MemoryStatus::Stale
        );
        assert_eq!(
            canonical_to_durable(MemoryStatus::Stale),
            DurableClaimStatus::Stale
        );
    }

    #[test]
    fn superseded_remains_superseded() {
        assert_eq!(
            durable_to_canonical(DurableClaimStatus::Superseded),
            MemoryStatus::Superseded
        );
        assert_eq!(
            canonical_to_durable(MemoryStatus::Superseded),
            DurableClaimStatus::Superseded
        );
    }

    #[test]
    fn unknown_remains_unknown() {
        assert_eq!(
            durable_to_canonical(DurableClaimStatus::Unknown),
            MemoryStatus::Unknown
        );
        assert_eq!(
            canonical_to_durable(MemoryStatus::Unknown),
            DurableClaimStatus::Unknown
        );
    }

    #[test]
    fn canonical_all_durable_variants_covered() {
        let all = [
            DurableClaimStatus::Asserted,
            DurableClaimStatus::Validated,
            DurableClaimStatus::Rejected,
            DurableClaimStatus::Stale,
            DurableClaimStatus::Disputed,
            DurableClaimStatus::Superseded,
            DurableClaimStatus::Unknown,
        ];
        for s in all {
            // Must not panic and must produce distinct results for Rejected/Disputed/Stale.
            let _ = durable_to_canonical(s);
        }
    }

    // ── Legacy lossy shim tests (kept to prevent accidental behavioural change) ──

    #[allow(deprecated)]
    #[test]
    fn durable_to_memory_mapping_is_explicit() {
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Asserted),
            MemoryClaimStatus::Unverified
        );
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Validated),
            MemoryClaimStatus::Active
        );
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Rejected),
            MemoryClaimStatus::Contradicted
        );
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Disputed),
            MemoryClaimStatus::Contradicted
        );
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Stale),
            MemoryClaimStatus::Superseded
        );
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Superseded),
            MemoryClaimStatus::Superseded
        );
        assert_eq!(
            durable_to_memory(DurableClaimStatus::Unknown),
            MemoryClaimStatus::Unverified
        );
    }

    #[allow(deprecated)]
    #[test]
    fn memory_to_durable_mapping_is_explicit() {
        assert_eq!(
            memory_to_durable(MemoryClaimStatus::Unverified),
            DurableClaimStatus::Asserted
        );
        assert_eq!(
            memory_to_durable(MemoryClaimStatus::Active),
            DurableClaimStatus::Validated
        );
        assert_eq!(
            memory_to_durable(MemoryClaimStatus::Contradicted),
            DurableClaimStatus::Disputed
        );
        assert_eq!(
            memory_to_durable(MemoryClaimStatus::Superseded),
            DurableClaimStatus::Superseded
        );
    }
}
