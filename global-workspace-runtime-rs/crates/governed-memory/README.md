# Governed Memory: CODEX-Native Bounded Memory Governance

**Status:** Integration Proof Candidate (not final freeze)

## Overview

The `governed-memory` crate provides bounded memory governance primitives that integrate with CODEX evidence, claim, contradiction, and audit systems. It salvages policy concepts from memvid-Human while maintaining CODEX runtime authority and hard invariants.

This is an **advisory layer** that improves memory admission, source trust evaluation, retrieval routing, and audit tracing — while keeping CODEX runtime, evidence vault, claim store, and contradiction engine authoritative.

## Key Design Principles

### 1. Authority Preservation

- **CODEX runtime-core** remains authoritative for action selection (10-action schema unchanged)
- **Evidence vault** is authoritative for verified input (immutable, hash-chained)
- **ClaimStore** is authoritative for the truth window (claim lifecycle)
- **Contradiction engine** is authoritative for conflict detection
- **Governed-memory is advisory** — provides decision support, not override

### 2. Hard Invariants (Enforced in Code & Tests)

- ✅ `real_external_executions == 0` (no actual tool calls)
- ✅ `local_provider_requests == 0` (no local inference)
- ✅ `cloud_provider_requests == 0` (no Azure OpenAI, Anthropic, etc.)
- ✅ `external_provider_requests == 0` (no Tavily, search APIs)
- ✅ No API keys stored anywhere
- ✅ No .mv2 storage activation
- ✅ No api_embed activation
- ✅ Provider output non-authoritative (advisory only)

### 3. "Belief" Terminology

In this crate, **"belief" means a structured memory assertion record**, not subjective consciousness:

> "An assertion conflict is detected not because an AI 'believes' something (implying consciousness), but because memory data structures record contradictory facts. The conflict handler marks these as disputed status, annotates with evidence links, and logs audit codes — all deterministic operations on data structures."

See `src/belief_conflict.rs` for assertion conflict handling (renamed from "resolver" for clarity).

## Module Overview

| Module | Purpose |
|--------|---------|
| `enums.rs` | Core classification enums: SourceTrustType, MemoryStatus, RetrievalIntentCategory, ConflictNature |
| `schemas.rs` | Bounded types: CandidateMemory, DurableMemory, Provenance, RetrievalQuery, RetrievalHit, PolicySet, GovernedClaimRecord |
| `reason_codes.rs` | Auditable decision codes (ADMISSION_VERIFIED_EVIDENCE, RETRIEVAL_AMBIGUOUS_MATCH, CONFLICT_DIRECT_CONTRADICTION, etc.) |
| `policy.rs` | Admission and retrieval policy rule evaluation |
| `source_trust.rs` | Source trust classification and confidence scoring (confidence affects score, NOT truth) |
| `admission.rs` | Memory admission gate: routes candidates to active claims, pending evidence, or rejection |
| `audit.rs` | Governance audit records for proof and compliance tracing |
| `retrieval_intent.rs` | Query intent analysis and routing to appropriate actions (retrieve, defer, ask, refuse) |
| `retrieval_planner.rs` | Retrieval strategy planning (read-only by design) |
| `belief_conflict.rs` | Assertion conflict handler: records disputes, marks claims as disputed |
| `codex_adapter.rs` | **Critical:** Bidirectional conversion between CODEX and governed-memory types; no provider metadata passes through |

## Usage Example

### Admit a Memory Candidate

```rust
use governed_memory::*;

let gate = MemoryAdmissionGate::default_policy();
let candidate = CandidateMemory {
    id: "mem_1".to_string(),
    subject: "weather".to_string(),
    predicate: "is_raining".to_string(),
    object: Some("true".to_string()),
    source_id: "evidence_vault_1".to_string(),
    source_type: SourceTrustType::VerifiedEvidence,
    confidence: 0.9,
    created_at: chrono::Utc::now(),
    evidence_id: Some("evidence_vault_1".to_string()),
    provenance: Provenance::DirectEvidence {
        evidence_id: "evidence_vault_1".to_string(),
        vault_entry_hash: "hash_abc".to_string(),
    },
};

let decision = gate.admit(&candidate);
assert!(decision.admitted);
assert_eq!(decision.storage_location, "active_claim");
```

### Route a Retrieval Query

```rust
use governed_memory::*;

let query = RetrievalQuery {
    query_id: "q_1".to_string(),
    query_text: "do I know the weather?".to_string(),
    context: None,
    intent_category: RetrievalIntentCategory::MemoryLookup,
    requires_verification: false,
    max_candidates: 10,
    confidence_threshold: 0.6,
    created_at: chrono::Utc::now(),
};

let decision = RetrievalRouter::route(&query);
assert_eq!(decision.recommended_action, "retrieve_memory");
// Runtime-core makes final action selection, not governed-memory
```

### Convert Evidence to Claim

```rust
use governed_memory::codex_adapter::*;

let candidate = evidence_entry_to_candidate(
    "evidence_1",
    "subject",
    "predicate",
    Some("object"),
    "hash_verified",
);
// This preserves the evidence hash for verification audit trail
```

## Admission Policy Rules

1. **Verified evidence** → admit as active claim (confidence 0.9)
2. **Trusted user input** → admit as active claim (confidence 0.7)  
3. **Low-confidence survey** → store as pending evidence only (NOT active claim)
4. **Duplicate evidence** → detect and reject (link to existing claim)
5. **Contradicting** → admit with disputed status, store metadata
6. **Retraction** → update target claim status (don't silent overwrite)
7. **Unsupported raw text** → reject (no evidence, no claim)

## Retrieval Intent Routing

Queries are routed to appropriate runtime actions based on intent:

| Intent | Action | Reason |
|--------|--------|--------|
| Memory lookup | `retrieve_memory` | User asking for a memory |
| Unsupported factual | `defer_insufficient_evidence` | Factual claim not backed by evidence |
| High-stakes low-evidence | `defer_insufficient_evidence` | Important claim with weak evidence (medical, financial) |
| Ambiguous | `ask_clarification` | Query is semantically ambiguous |
| Provider-gated | `refuse_unsafe` | Would require external tool/provider (policy-blocked) |

**Important:** Router *recommends* action. Runtime-core *selects* action. Governed-memory is advisory only.

## Test Coverage

53 unit + integration tests covering:

- ✅ Admission verified evidence promotes claim
- ✅ Low-trust observation stores evidence only
- ✅ Unsupported input does not create active claim
- ✅ Duplicate evidence does not duplicate claim
- ✅ Source trust affects confidence but does not prove truth
- ✅ Retrieve is read-only by default
- ✅ Conflict creates disputed/contradicted status
- ✅ Audit records reason codes
- ✅ No provider metadata in conversions
- ✅ CODEX runtime still owns selected_action
- ✅ No .mv2 active path
- ✅ No api_embed active path
- ✅ No provider/network paths

## Integration with CODEX

### Evidence Vault → Governed Memory

```
EvidenceEntry
  ↓ (hash-verified, via evidence_entry_to_candidate)
CandidateMemory
  ↓ (admission policy evaluated)
MemoryAdmissionDecision (Active | Pending | Rejected)
  ↓ (if Active, promoted via claim_to_governed_record)
GovernedClaimRecord (with audit_reasons, contradictions)
  ↓ (stored in ClaimStore; authority remains there)
ClaimStore (authoritative truth window)
```

### Contradiction Engine → Governed Memory

```
Contradiction detected (via contradiction_to_governed_conflict)
  ↓
GovernedConflictMetadata (records evidence on both sides)
  ↓
Marks claim status = Disputed
  ↓
NOT auto-resolved; contradictions remain visible
```

### Retrieval Query → Action Selection

```
RetrievalQuery
  ↓ (intent analysis via RetrievalRouter)
RetrievalDecision (recommended_action + reason_codes)
  ↓
Runtime-core selects final action (may differ from recommendation)
  ↓
One of 10 CODEX actions executed
```

## Dependencies

- **serde** (1.x) — JSON serialization
- **serde_json** (1.x) — JSON parsing
- **chrono** (0.4) — Timestamps
- **thiserror** (1) — Error types
- **anyhow** (1) — Fallback error propagation
- **Internal:** evidence, contradiction, memory, runtime-core crates

NO external service dependencies (no reqwest, no openai-api, no embedding libraries).

## Proof & Verification

Proof artifacts are generated by `runtime-cli proof --strict --long-horizon --nl --out ../artifacts/proof/current`.

Governed-memory contributes to:
- `reasoning_audit_report.json` — admission, retrieval, conflict decisions
- `contradiction_integration_report.json` — conflict detection (authority: contradiction crate)
- `evidence_claim_link_report.json` — evidence-claim mappings
- `tool_policy_report.json` — verification that tools remain policy-gated

**Invariant validation scripts:**
- `check_action_types.py` — Verify 10-action schema unchanged
- `check_sentience_claims.py` — Verify no consciousness/AGI claims
- `check_no_mv2.py` — Verify no .mv2 storage activation  
- `check_resource_recovery.py` — Verify no orphaned handles

## Not Included (Intentionally Excluded)

- ❌ Memvid storage (.mv2, mvz, etc.)
- ❌ Embedding/vector search (api_embed, text_embed, clip, etc.)
- ❌ Provider execution (OpenAI, Anthropic, Tavily, etc.)
- ❌ Network access (HTTP clients, search APIs)
- ❌ API key storage
- ❌ Provider-based extraction (Whisper, OCR via cloud service)
- ❌ Sentience/consciousness claims
- ❌ Human memory equivalence claims
- ❌ Production-ready memory system claims

This is bounded governance, not full-featured memory management.

## Transitions to CODEX

All governance decisions flow into or feed from CODEX authoritative sources:

1. **Admission** → ClaimStore (truth window authority)
2. **Retrieval** → Runtime-core (action selection authority)
3. **Contradiction** → Contradiction engine (conflict detection authority)
4. **Audit** → Archive (via JSONL backend; authority: ArchiveBackend)

No alternative truth sources. No parallel decision-making. No provider overrides.

## Future Extensions (Not in Phase 1)

- Aggregation rules for multiple evidence sources per fact
- Custom policy rule callbacks (currently placeholder)
- Advanced retrieval ranking beyond exact/semantic/history matches
- Performance optimization for large claim stores (indexing, caching — read-only)
- Dispute resolution templates (currently records disputes without auto-resolution)

## License

MIT (matching CODEX workspace)

## Status

**Integration Proof Candidate** — This module is ready for integration with CODEX. Proof generation and validation are passing all invariant checks. This is NOT a final freeze; further refinement or optimization may be needed before production use.

---

**For questions or integration, see:**
- CODEX ARCHITECTURE.md
- `crates/memory/src/claim_store.rs` (ClaimStore authority)
- `crates/evidence/src/lib.rs` (Evidence vault)
- `crates/contradiction/src/lib.rs` (Contradiction detection)
- `crates/runtime-core/src/action.rs` (10-action schema)
