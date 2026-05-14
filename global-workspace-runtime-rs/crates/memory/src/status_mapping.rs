//! Explicit mapping between durable claim statuses and in-memory lifecycle statuses.

use crate::durable_memory_provider::ClaimStatus as DurableClaimStatus;
use crate::ClaimStatus as MemoryClaimStatus;

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
