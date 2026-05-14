# Provider & Tool Policy Gates

**Defines how external systems are controlled and disabled by design.**

---

## Core Design: Deny by Default

### **Principle**

All provider and tool execution paths must:

1. **Default to denied** (no execution)
2. **Require explicit policy override** (per-path approval)
3. **Check the same approval gate** (no bypass paths)
4. **Increment counters** (all decisions audited)
5. **Emit audit events** (decision recorded)

---

## Provider Policy

### **Provider Candidates**

Candidates are generated during the runtime cognitive pipeline:

1. Observation received → interpretation
2. Memory retrieval → candidate generation
3. Candidate scoring → symbolic activation
4. Critic evaluation → provider candidates considered

**At critic gate:**
- Provider candidates checked against `provider_gate` policy
- If denied: marked as "provider_denied_by_policy"
- If denied: increments `provider_policy_report.provider_policies_checked` and `provider_policies_denied`
- If denied: emits `ProviderPolicyCheckedEvent` with `decision: Denied`
- If denied: does NOT execute provider call
- If approved (rare): would execute if feature-enabled and gate enabled

### **Provider Approval Gate**

**Located:** `ui/codex-dioxus/src/bridge/runtime_client.rs::ProviderGate`

**Default State:** `deny_all`

**Approval Levels:**

```rust
pub enum ProviderApprovalLevel {
    DenyAll,           // ❌ No provider execution, ever
    LocalhostOnly,     // ⚠️ Experimental: localhost-only (Ollama, Turboquant)
    CloudApproved,     // 🔴 NEVER enabled in default builds (would approve cloud providers)
}
```

**Current Default:** `DenyAll` (hardcoded)

**How to Change (if you must test provider locally):**

```rust
// ❌ DON'T this way (breaks default safety):
let provider_gate = ProviderGate::new(ProviderApprovalLevel::LocalhostOnly);
// (This is a development/test override, never in production)

// ✅ Instead, use feature flag (tested path):
#[cfg(feature = "ui-local-providers")]
let provider_gate = ProviderGate::new(ProviderApprovalLevel::LocalhostOnly);
// (Feature `ui-local-providers` is off by default)
```

### **Provider Types**

| Provider | Mode | Default | Feature Flag | Approval Level Required |
|---|---|---|---|---|
| Ollama (localhost) | Read-only | Disabled | `ui-local-providers` | `LocalhostOnly` |
| Turboquant (localhost) | Read-only | Disabled | `ui-local-providers` | `LocalhostOnly` |
| Cloud (OpenAI, Anthropic, etc.) | API calls | **Disabled** | None | (Never approved) |
| External APIs | HTTP calls | **Disabled** | None | (Never approved) |

**Important:** No cloud provider is ever approved even if you change `ProviderApprovalLevel`. The proof explicitly denies all external providers.

### **How Providers Are Disabled**

1. **No API keys stored** — No hard-coded keys in code, no env vars in proof command
2. **No endpoint configuration** — Cloud endpoints not configured
3. **Policy gate checked early** — Before any HTTP call or network init
4. **Feature-gated paths** — Localhost requires feature flag (off by default)
5. **Architecture guard check** — `check_sentience_claims.py` also verifies no API keys found

### **Provider Counters**

**Tracked in:** `provider_policy_report.json`

```json
{
  "provider_policy_report": {
    "provider_policies_checked": N,        // Total candidates evaluated for provider use
    "provider_policies_approved": 0,       // Currently: always 0 (no approval in proof)
    "provider_policies_denied": N,         // Denied due to gate or policy
    "provider_execution_requests": 0,      // Provider HTTP/API calls attempted
    "provider_execution_failures": 0,      // Provider calls that failed (N/A, all denied)
    "provider_execution_timeouts": 0,      // Provider call timeouts (N/A, all denied)
    "provider_locally_executed": 0,        // Localhost provider executions (Ollama, Turboquant)
    "provider_cloud_execution_attempted": 0  // Cloud provider calls (should always be 0)
  }
}
```

**Current state (proof run):** All zeros except `provider_policies_checked` (equals number of candidate generations).

---

## Tool Policy

### **Tool Candidates**

Tool candidates are generated during cognitive pipeline:

1. Action candidates suggested by planner
2. Tool actions identified from action types
3. Critic evaluates each tool candidate

**At critic gate:**
- Tool candidates checked against `tool_policy` gate
- If denied: marked as "tool_denied_by_policy"
- If denied: increments `tool_policy_report.tool_policies_denied`
- If denied: emits `ToolPolicyCheckedEvent` with `decision: Denied`
- If denied: does NOT execute any external system call
- If approved (default): marked as "tool_dry_run" (simulated, not executed)

### **Tool Approval Gate**

**Located:** `global-workspace-runtime-rs/crates/tools/src/lib.rs::ToolExecutionMode`

**Default State:** `DryRunOnly`

**Execution Modes:**

```rust
pub enum ToolExecutionMode {
    Deny,        // ❌ All tools refused (most restrictive)
    DryRun,      // ✅ Tools simulated, not executed (default, safe)
    Approve,     // 🔴 Tools actually executed (NEVER in proof or default UI)
}
```

**Current Default:** `DryRun` (toolexecution simulated but not real)

---

### **Tool Lifecycle**

1. **Candidate Generation** — Tool suggested by planner
2. **Policy Check** — Tool mode checked (Deny/DryRun/Approve)
3. **Critic Scoring** — Tool scored if not denied
4. **Action Selection** — If selected and DryRun: marked with `dry_run: true`
5. **Result Simulation** — Fake result generated (simulated output)
6. **Audit** — Tool decision recorded (whether executed or simulated)

### **Tool Execution Policy Examples**

#### **Example 1: API Tool (Denied)**

```
User: "Call the weather API and tell me tomorrow's forecast"

Pipeline:
1. Planner suggests: execute_bounded_tool(action: call_weather_api)
2. Critic checks policy: tool_policy.mode == DryRun
3. Decision: execute_bounded_tool (dry-run mode)
4. Result: "dry_run: true, simulated_output: {temperature: 72F, ...}"
5. Answer: "I simulated calling the weather API (dry-run only). In dry-run, the result is: ..."
6. Audit: ToolPolicyCheckedEvent { tool_id: "weather_api", decision: DryRun, executed: false }
```

#### **Example 2: Filesystem Tool (Denied)**

```
User: "Delete all .tmp files"

Pipeline:
1. Planner suggests: execute_bounded_tool(action: delete_file)
2. Critic checks policy: tool_policy.mode == DryRun
3. Decision: execute_bounded_tool (dry-run mode)
4. Result: "dry_run: true, simulated_output: {deleted_count: 0, reason: 'dry_run mode'}"
5. Answer: "I would have deleted X files (dry-run only). No actual deletion occurred."
6. Audit: ToolPolicyCheckedEvent { tool_id: "delete_file", decision: DryRun, executed: false }
```

### **Tool Counters**

**Tracked in:** `tool_policy_report.json`

```json
{
  "tool_policy_report": {
    "tool_policies_checked": N,            // Total tool candidates evaluated
    "tool_policies_approved": N,           // Approved for dry-run (DryRun mode)
    "tool_policies_denied": M,             // Denied (Deny mode or unsafe)
    "tool_execution_attempts_dry_run": N,  // Simulated tool calls (approved for dry-run)
    "tool_execution_attempts_real": 0,     // Real tool calls (should always be 0)
    "tool_execution_failures_real": 0,     // Real tool call failures (should always be 0)
    "tool_results_simulated": N,           // Fake results generated for dry-run calls
    "tool_execution_denied_by_policy": M   // Tools refused entirely
  }
}
```

**Current state (proof run):** Real execution always 0. All approved tools in dry-run mode.

---

## Guarded Execution Principle

### **All Provider/Tool Paths Must Share One Gate**

**Bad Architecture (Bypass Risk):**
```rust
// ❌ Two independent paths
if send_user_message_stream() {
    check_provider_gate()  // Gate check here
    call_provider()
} else {
    call_provider()  // ⚠️ Gate check missing! Bypass!
}
```

**Good Architecture (Single Gate):**
```rust
// ✅ Single guarded entry point
async fn guarded_provider_response(
    provider_kind: ProviderKind,
    input: &str,
    provider_gate: &ProviderGate,
    counters: &mut ProviderCounters
) -> Result<ProviderResponse, ProviderDenied> {
    // Check gate once
    if !provider_gate.is_enabled_for(provider_kind) {
        counters.increment_denied();
        return Err(ProviderDenied { reason: "policy gate denies this provider" });
    }
    // All branches go through here
    call_provider(provider_kind, input).await
}

// Both paths use the guarded function
impl RuntimeClient {
    pub async fn send_user_message(&self, message: &str) 
        -> Result<RuntimeStepResult, RuntimeError> 
    {
        // ... preparation ...
        guarded_provider_response(ProviderKind::Ollama, message, &self.provider_gate, &mut self.counters).await
    }
    
    pub async fn send_user_message_stream(&self, message: &str)
        -> Result<impl Stream<Item = RuntimeStepResult>, RuntimeError>
    {
        // ... preparation ...
        guarded_provider_response(ProviderKind::Ollama, message, &self.provider_gate, &mut self.counters).await
    }
}
```

---

## Audit Trail: Policy Decisions

### **Events Emitted**

Every provider or tool policy decision emits an audit event:

```rust
pub enum RuntimeEvent {
    // ... other variants ...
    ProviderPolicyCheckedEvent {
        timestamp: SystemTime,
        provider_kind: ProviderKind,
        approval_level: ProviderApprovalLevel,
        decision: PolicyDecision,  // Approved | Denied
        reason: String,
        request_id: String,
    },
    ToolPolicyCheckedEvent {
        timestamp: SystemTime,
        tool_id: String,
        execution_mode: ToolExecutionMode,
        decision: PolicyDecision,  // Approved | Denied
        reason: String,
        dry_run_result: Option<ToolResult>,
    },
}
```

### **Audit Log Example**

```json
[
  {
    "event_type": "ProviderPolicyCheckedEvent",
    "timestamp": "2026-05-13T20:17:00Z",
    "provider_kind": "Ollama",
    "approval_level": "DenyAll",
    "decision": "Denied",
    "reason": "provider_gate policy level: DenyAll"
  },
  {
    "event_type": "ToolPolicyCheckedEvent",
    "timestamp": "2026-05-13T20:17:01Z",
    "tool_id": "weather_api",
    "execution_mode": "DryRun",
    "decision": "Approved",
    "reason": "approved for dry-run simulation",
    "dry_run_result": {
      "temperature": 72,
      "conditions": "sunny",
      "source": "simulated"
    }
  }
]
```

---

## Safety Guarantees

### **Proven in Proof Artifacts**

1. ✅ **Provider counters:** All provider execution requests are 0 (none attempted)
2. ✅ **Tool real execution:** All real tool execution counts are 0 (all dry-run or denied)
3. ✅ **Audit events:** All policy decisions recorded and traceable
4. ✅ **Replay idempotence:** Same policy decisions on replay
5. ✅ **Feature gate containment:** Localhost-only paths only enabled via feature flag

### **Impossible to Break Without Modifying Code**

- Cannot pass `None` as `provider_gate` — would fail type check
- Cannot skip audit event emission — reducer enforces it
- Cannot call provider without gate check — guarded function enforces it
- Cannot enable provider in default build — feature flag default is off
- Cannot store API keys — no environment or file-based key handling

---

## Development Scenarios

### **Scenario A: I Want to Test Locally with Ollama**

**Step 1: Build with feature flag**
```bash
cd ui/codex-dioxus
cargo build --features "ui-local-providers"
```

**Step 2: Understand the risk**
- Ollama provider will be available
- Still requires explicit gate override at runtime (code change)
- This is development-only, not production

**Step 3: Override gate (development only)**
```rust
// In runtime_client.rs, development branch only:
#[cfg(feature = "ui-local-providers")]
let provider_gate = ProviderGate::new(ProviderApprovalLevel::LocalhostOnly);
// (Not in default proof or production)
```

**Step 4: Test locally**
- Ollama running on localhost:11434
- Provider calls go through guarded function
- Counters incremented
- Audit trail recorded

**Step 5: Revert for proof**
```bash
# For proof, use default (feature off):
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
# (ui-local-providers is off, provider_gate is DenyAll)
```

### **Scenario B: I Want to Add a New Tool**

**Step 1: Define tool**
```rust
pub enum BoundedTool {
    WeatherApi,
    NewsApi,
    NewTool,  // Add here
}
```

**Step 2: Implement tool candidate logic**
```rust
impl ToolCandidateGenerator {
    fn generate(&self, action: &ActionType) -> Vec<ToolCandidate> {
        // Suggest your new tool if action matches
    }
}
```

**Step 3: Add to critic scorer**
```rust
impl Critic {
    fn score_tool(&self, tool: &ToolCandidate, context: &RuntimeContext) -> Score {
        // Score your tool
    }
}
```

**Step 4: Add to policy gate**
```rust
impl ToolExecutionMode {
    fn check_policy(&self, tool_id: &str) -> PolicyDecision {
        match self {
            DryRun => PolicyDecision::Approved,  // (tool will be simulated)
            Deny => PolicyDecision::Denied,
            Approve => PolicyDecision::Approved, // (should never happen)
        }
    }
}
```

**Step 5: Implement dry-run result**
```rust
fn dry_run_tool_call(&self, tool: &ToolCandidate) -> ToolResult {
    match tool.id {
        "weather_api" => fake_weather_result(),
        "news_api" => fake_news_result(),
        "new_tool" => fake_new_tool_result(),  // Add here
    }
}
```

**Step 6: Test**
```bash
# Tool candidates should be generated
# Tool policy should be checked
# Tool result should be simulated
# Counters incremented
# Audit event emitted

cargo test tools --
cargo test cognition --  # If integrated with planner
```

**Step 7: Verify in proof**
```bash
cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current
# tool_policy_report.json should show:
# - tool_policies_checked: incremented if your tool is a candidate
# - tool_policies_approved: incremented if checked and not denied
# - tool_execution_attempts_real: still 0 (all dry-run)
```

---

## Do's and Don'ts

### **✅ Do**

- ✅ Use provider_gate before any provider call
- ✅ Use tool_policy gate before any real tool call
- ✅ Count all decisions (approved and denied)
- ✅ Emit audit events
- ✅ Dry-run tools in proof
- ✅ Block all real provider execution
- ✅ Feature-gate localhost providers
- ✅ Document policy changes
- ✅ Test gate enforcement

### **❌ Don't**

- ❌ Skip gate checks for convenience
- ❌ Create provider paths outside guarded function
- ❌ Store API keys anywhere
- ❌ Execute real tools in proof
- ❌ "Just test" with production endpoints
- ❌ Feature-gate safety (safety is default)
- ❌ Claim policies are enforced without testing
- ❌ Deploy with policy gates disabled
- ❌ Assume feature flag OFF is tested (test both on and off)

---

## Next Steps

- See `docs/PROVIDER_TOOL_POLICY.md` (you are reading this) for policy implementation
- See `docs/MEMORY_CLAIM_MODEL.md` for memory/claim integration with policy
- See `docs/UI_RUNTIME_BRIDGE.md` for safe UI integration patterns
- See Phase 2 in `docs/INTEGRATION_IMPLEMENTATION_ROADMAP.md` for provider gate unification work
