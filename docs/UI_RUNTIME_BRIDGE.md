# UI Runtime Bridge: Safe Integration Patterns

**Defines how Dioxus UI safely calls Rust runtime without compromising safety boundaries.**

---

## Core Architecture

### **One-Way Dependency**

```
┌──────────────────┐
│  Dioxus UI       │
│  (codex-dioxus)  │
│                  │
│  ┌────────────┐  │
│  │  Bridge    │  │
│  │  (types.rs,│  │ ──calls──>  ┌──────────────────┐
│  │ runner.rs) │  │             │  Rust Runtime    │
│  └────────────┘  │             │ (runtime-core)   │
│                  │             │                  │
└──────────────────┘             │  ✅ Authority    │
                                  │  ✅ Determinist │
                                  │  ✅ Auditable   │
                                  └──────────────────┘
```

**Key Rule:** Rust runtime ≠ calls UI. UI calls runtime.

**Why:** Rust runtime is authoritative; UI is presentation layer only.

---

## Bridge Module Structure

**Location:** `ui/codex-dioxus/src/bridge/`

| File | Purpose |
|---|---|
| `mod.rs` | Module definition, re-exports |
| `runtime_client.rs` | RuntimeClient, provider gate, UI modes, bridges to runtime |
| `types.rs` | Bridge type definitions (shared types between UI and runtime) |
| `proof_reader.rs` | Read proof artifacts for display |

---

## UI Runtime Modes

### **Mode 1: MockUiMode** (Development/Testing)

```rust
pub enum RuntimeBridgeMode {
    MockUiMode {
        deterministic_actions: Vec<ActionType>,
        error_injection: Option<RuntimeError>,
    },
}
```

**Properties:**
- No external calls (Rust runtime not called)
- Deterministic: returns pre-set action sequence
- Use case: UI development, testing without runtime

**Safety:** ✅ No side effects, no network calls

### **Mode 2: LocalCodexRuntimeReadOnly** (Default, Safe)

```rust
pub enum RuntimeBridgeMode {
    LocalCodexRuntimeReadOnly {
        runtime: RuntimeHandle,  // In-process runtime
        provider_gate: ProviderGate,  // Policy gate
    },
}
```

**Properties:**
- Calls real Rust runtime (runtime-core)
- No external provider calls
- No real tool execution (dry-run only)
- Deterministic replay available
- Read-only except for internal state updates

**Safety:** ✅ No external execution, provider gate enforced

**How It Works:**
```rust
pub async fn send_user_message(&self, message: &str) 
    -> Result<RuntimeStepResult, RuntimeError>
{
    match self.mode {
        LocalCodexRuntimeReadOnly { runtime, provider_gate } => {
            // Check provider gate (it's DenyAll by default)
            if !provider_gate.is_enabled() {
                return Ok(RuntimeStepResult {
                    selected_action: ActionType::no_op,
                    provider_attempted: false,
                    ..Default::default()
                });
            }
            
            // Call runtime
            let result = runtime.step(
                RuntimeInput { text: message, .. },
                StepContext { provider_gate, .. }
            ).await?;
            
            Ok(result)
        },
        // Other modes...
    }
}
```

### **Mode 3: LocalOllamaProvider** (Experimental, Feature-Gated)

```rust
#[cfg(feature = "ui-local-providers")]
pub enum RuntimeBridgeMode {
    LocalOllamaProvider {
        runtime: RuntimeHandle,
        provider_gate: ProviderGate,
        ollama_endpoint: String,  // localhost:11434
    },
}
```

**Properties:**
- Calls Rust runtime
- **May** call Ollama (localhost:11434) if provider gate enables it
- Feature-gated: `ui-local-providers` (off by default)
- Still uses policy gate (can be overridden for testing)

**Safety:** ⚠️ Feature-gated, localhost-only, still requires gate override

**Development Only:** Never use in production or proof runs.

### **Mode 4: LocalTurboquantProvider** (Experimental, Feature-Gated)

```rust
#[cfg(feature = "ui-local-providers")]
pub enum RuntimeBridgeMode {
    LocalTurboquantProvider {
        runtime: RuntimeHandle,
        provider_gate: ProviderGate,
        turboquant_endpoint: String,  // localhost:port
    },
}
```

**Properties:**
- Same as Ollama but calls Turboquant
- Feature-gated: `ui-local-providers`
- Development only

**Safety:** ⚠️ Feature-gated, localhost-only

### **Mode 5: ExternalProviderDisabled** (Explicit Denial)

```rust
pub enum RuntimeBridgeMode {
    ExternalProviderDisabled {
        runtime: RuntimeHandle,
        reason: String,  // Why disabled
    },
}
```

**Properties:**
- Calls Rust runtime
- All provider gates return Disabled
- No external calls possible (even if you tried)

**Safety:** ✅ Explicitly impossible to call external providers

---

## Provider Gate: Core Safety Mechanism

### **ProviderGate Definition**

```rust
pub struct ProviderGate {
    approval_level: ProviderApprovalLevel,
}

pub enum ProviderApprovalLevel {
    DenyAll,           // ❌ No providers ever
    LocalhostOnly,     // ⚠️ Localhost providers only (Ollama, Turboquant)
    CloudApproved,     // 🔴 Would allow cloud (never used in proof/default)
}
```

### **ProviderGate Methods**

```rust
impl ProviderGate {
    pub fn new(level: ProviderApprovalLevel) -> Self { /* ... */ }
    
    pub fn is_enabled(&self) -> bool {
        matches!(self.approval_level, 
            ProviderApprovalLevel::LocalhostOnly 
            | ProviderApprovalLevel::CloudApproved)
    }
    
    pub fn is_enabled_for(&self, provider: ProviderKind) -> bool {
        match self.approval_level {
            DenyAll => false,
            LocalhostOnly => matches!(provider, 
                ProviderKind::Ollama | ProviderKind::Turboquant),
            CloudApproved => true,  // (never reached in proof)
        }
    }
    
    pub fn deny_all() 
