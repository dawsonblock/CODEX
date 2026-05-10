//! Policy-gated tool scaffold — **enforced at runtime via RuntimeLoop critic gate.**
//!
//! The `ToolGate` evaluates whether a tool is permitted before
//! `execute_bounded_tool` can be selected. Policies define:
//! - which tools are allowed
//! - max consecutive executions
//! - whether confirmation or sandboxing is required
//!
//! Tools are only executable when the planner selects `execute_bounded_tool`
//! AND tool-specific policy permits it. No tool can execute without
//! explicit policy approval.
//!
//! # Honesty boundaries
//!
//! - Tools are NOT safe. They are policy-gated.
//! - This is NOT arbitrary tool execution. It is bounded and gated.
//! - Tool execution is NOT autonomous. Policy must approve each use.

use serde::{Deserialize, Serialize};

/// What a tool can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapability {
    pub tool_id: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub side_effects: Vec<String>,
}

/// Policy governing a tool's execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    pub tool_id: String,
    pub allowed_actions: Vec<String>,
    pub max_consecutive: usize,
    pub requires_confirmation: bool,
    pub sandbox_required: bool,
}

/// Result of a policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub permitted: bool,
    pub reason: String,
    pub tool_id: String,
}

/// Record of a tool execution attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRecord {
    pub tool_id: String,
    pub cycle_id: u64,
    pub inputs: serde_json::Value,
    pub outputs: Option<serde_json::Value>,
    pub policy_permitted: bool,
    pub error: Option<String>,
    pub timestamp: String,
}

/// Policy violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicyViolation {
    pub tool_id: String,
    pub violation_type: ViolationType,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    NotInAllowedList,
    MaxConsecutiveExceeded,
    ConfirmationRequired,
    SandboxRequired,
    PolicyBypass,
}

/// Request to evaluate a tool execution against policy.
#[derive(Debug, Clone)]
pub struct EvaluationRequest {
    pub tool_id: String,
    pub action: String,
    pub cycle_id: u64,
    pub confirmation_present: bool,
    pub sandbox_active: bool,
}

impl EvaluationRequest {
    pub fn new(tool_id: impl Into<String>, action: impl Into<String>, cycle_id: u64) -> Self {
        Self {
            tool_id: tool_id.into(),
            action: action.into(),
            cycle_id,
            confirmation_present: false,
            sandbox_active: false,
        }
    }

    pub fn with_confirmation(mut self, present: bool) -> Self {
        self.confirmation_present = present;
        self
    }

    pub fn with_sandbox(mut self, active: bool) -> Self {
        self.sandbox_active = active;
        self
    }
}

/// The tool gate — evaluates policy before allowing execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolGate {
    policies: Vec<ToolPolicy>,
    execution_history: Vec<ToolExecutionRecord>,
    violations: Vec<ToolPolicyViolation>,
    /// When true, all tools run in dry-run mode (no side effects).
    pub dry_run: bool,
    /// Allowlisted tool IDs that bypass policy checks.
    pub allowlist: Vec<String>,
}

impl ToolGate {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            execution_history: Vec::new(),
            violations: Vec::new(),
            dry_run: false,
            allowlist: Vec::new(),
        }
    }

    /// Check if a tool is in the allowlist (bypasses policy).
    pub fn is_allowlisted(&self, tool_id: &str) -> bool {
        self.allowlist.iter().any(|id| id == tool_id)
    }

    /// Add a tool to the allowlist.
    pub fn allowlist_add(&mut self, tool_id: impl Into<String>) {
        self.allowlist.push(tool_id.into());
    }

    /// Register a tool policy.
    pub fn register_policy(&mut self, policy: ToolPolicy) {
        self.policies.push(policy);
    }

    /// Evaluate whether a tool is permitted to execute.
    /// Enforces: registration, allowed_actions, max_consecutive,
    /// requires_confirmation, sandbox_required.
    /// Allowlisted tools bypass all checks. Dry-run mode permits but flags.
    pub fn evaluate(&mut self, request: &EvaluationRequest) -> PolicyEvaluation {
        let tid = &request.tool_id;

        // Allowlisted tools bypass all policy checks
        if self.is_allowlisted(tid) {
            return PolicyEvaluation {
                permitted: true,
                reason: "allowlisted".into(),
                tool_id: tid.clone(),
            };
        }

        // Dry-run mode: permit execution but flag as dry-run
        if self.dry_run {
            return PolicyEvaluation {
                permitted: true,
                reason: "dry_run_mode".into(),
                tool_id: tid.clone(),
            };
        }

        let policy = match self.policies.iter().find(|p| p.tool_id == *tid) {
            Some(p) => p,
            None => {
                let violation = ToolPolicyViolation {
                    tool_id: tid.clone(),
                    violation_type: ViolationType::NotInAllowedList,
                    context: format!("no policy registered for {tid}"),
                };
                self.violations.push(violation);
                return PolicyEvaluation {
                    permitted: false,
                    reason: "no policy registered".into(),
                    tool_id: tid.clone(),
                };
            }
        };

        // Check allowed_actions
        if !policy.allowed_actions.is_empty()
            && !policy.allowed_actions.iter().any(|a| a == &request.action)
        {
            let violation = ToolPolicyViolation {
                tool_id: tid.clone(),
                violation_type: ViolationType::NotInAllowedList,
                context: format!(
                    "action '{}' not in allowed_actions {:?}",
                    request.action, policy.allowed_actions
                ),
            };
            self.violations.push(violation);
            return PolicyEvaluation {
                permitted: false,
                reason: format!("action '{}' not in allowed list", request.action),
                tool_id: tid.clone(),
            };
        }

        // Check confirmation
        if policy.requires_confirmation && !request.confirmation_present {
            let violation = ToolPolicyViolation {
                tool_id: tid.clone(),
                violation_type: ViolationType::ConfirmationRequired,
                context: "confirmation required but not present".into(),
            };
            self.violations.push(violation);
            return PolicyEvaluation {
                permitted: false,
                reason: "confirmation required".into(),
                tool_id: tid.clone(),
            };
        }

        // Check sandbox
        if policy.sandbox_required && !request.sandbox_active {
            let violation = ToolPolicyViolation {
                tool_id: tid.clone(),
                violation_type: ViolationType::SandboxRequired,
                context: "sandbox required but not active".into(),
            };
            self.violations.push(violation);
            return PolicyEvaluation {
                permitted: false,
                reason: "sandbox required".into(),
                tool_id: tid.clone(),
            };
        }

        // Check max consecutive
        let consecutive = self
            .execution_history
            .iter()
            .rev()
            .take(policy.max_consecutive)
            .filter(|r| r.tool_id == *tid && r.cycle_id == request.cycle_id.saturating_sub(1))
            .count();

        if consecutive >= policy.max_consecutive && policy.max_consecutive > 0 {
            let violation = ToolPolicyViolation {
                tool_id: tid.clone(),
                violation_type: ViolationType::MaxConsecutiveExceeded,
                context: format!("{consecutive} >= {}", policy.max_consecutive),
            };
            self.violations.push(violation);
            return PolicyEvaluation {
                permitted: false,
                reason: format!("max consecutive ({}) exceeded", policy.max_consecutive),
                tool_id: tid.clone(),
            };
        }

        PolicyEvaluation {
            permitted: true,
            reason: "policy satisfied".into(),
            tool_id: tid.clone(),
        }
    }

    /// Record a tool execution.
    pub fn record_execution(
        &mut self,
        tool_id: &str,
        cycle_id: u64,
        inputs: serde_json::Value,
        outputs: Option<serde_json::Value>,
        error: Option<String>,
    ) {
        let record = ToolExecutionRecord {
            tool_id: tool_id.into(),
            cycle_id,
            inputs,
            outputs,
            policy_permitted: true,
            error,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.execution_history.push(record);
    }

    /// Record a blocked execution.
    pub fn record_block(&mut self, tool_id: &str, cycle_id: u64, reason: &str) {
        let record = ToolExecutionRecord {
            tool_id: tool_id.into(),
            cycle_id,
            inputs: serde_json::Value::Null,
            outputs: None,
            policy_permitted: false,
            error: Some(reason.into()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.execution_history.push(record);
    }

    /// All violations recorded.
    pub fn violations(&self) -> &[ToolPolicyViolation] {
        &self.violations
    }

    /// Execution history.
    pub fn history(&self) -> &[ToolExecutionRecord] {
        &self.execution_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_policy_permits_tool() {
        let mut gate = ToolGate::new();
        gate.register_policy(ToolPolicy {
            tool_id: "read_file".into(),
            allowed_actions: vec!["read".into()],
            max_consecutive: 10,
            requires_confirmation: false,
            sandbox_required: false,
        });

        let req = EvaluationRequest::new("read_file", "read", 1);
        let eval = gate.evaluate(&req);
        assert!(eval.permitted);
    }

    #[test]
    fn unregistered_tool_is_blocked_mut() {
        let mut gate = ToolGate::new();
        let req = EvaluationRequest::new("unknown_tool", "read", 1);
        let eval = gate.evaluate(&req);
        assert!(!eval.permitted);
    }

    #[test]
    fn disallowed_action_is_blocked() {
        let mut gate = ToolGate::new();
        gate.register_policy(ToolPolicy {
            tool_id: "read_file".into(),
            allowed_actions: vec!["read".into()],
            max_consecutive: 10,
            requires_confirmation: false,
            sandbox_required: false,
        });
        let req = EvaluationRequest::new("read_file", "delete", 1);
        let eval = gate.evaluate(&req);
        assert!(!eval.permitted);
    }

    #[test]
    fn missing_confirmation_is_blocked() {
        let mut gate = ToolGate::new();
        gate.register_policy(ToolPolicy {
            tool_id: "dangerous_tool".into(),
            allowed_actions: vec!["execute".into()],
            max_consecutive: 10,
            requires_confirmation: true,
            sandbox_required: false,
        });
        let req = EvaluationRequest::new("dangerous_tool", "execute", 1);
        let eval = gate.evaluate(&req);
        assert!(!eval.permitted);
    }

    #[test]
    fn valid_request_allowed() {
        let mut gate = ToolGate::new();
        gate.register_policy(ToolPolicy {
            tool_id: "safe_tool".into(),
            allowed_actions: vec!["read".into()],
            max_consecutive: 10,
            requires_confirmation: false,
            sandbox_required: false,
        });
        let req = EvaluationRequest::new("safe_tool", "read", 1);
        let eval = gate.evaluate(&req);
        assert!(eval.permitted);
    }

    #[test]
    fn max_consecutive_enforced() {
        let mut gate = ToolGate::new();
        gate.register_policy(ToolPolicy {
            tool_id: "write_file".into(),
            allowed_actions: vec!["write".into()],
            max_consecutive: 1,
            requires_confirmation: false,
            sandbox_required: false,
        });

        // First execution OK
        gate.record_execution(
            "write_file",
            0,
            serde_json::json!({"path": "a.txt"}),
            None,
            None,
        );

        // Second execution should be blocked
        let req = EvaluationRequest::new("write_file", "write", 1);
        let eval = gate.evaluate(&req);
        assert!(!eval.permitted);
    }

    #[test]
    fn violations_are_recorded() {
        let mut gate = ToolGate::new();
        let req = EvaluationRequest::new("no_policy", "read", 1);
        gate.evaluate(&req);
        assert_eq!(gate.violations().len(), 1);
    }

    #[test]
    fn missing_sandbox_is_blocked() {
        let mut gate = ToolGate::new();
        gate.register_policy(ToolPolicy {
            tool_id: "sandboxed_tool".into(),
            allowed_actions: vec!["execute".into()],
            max_consecutive: 10,
            requires_confirmation: false,
            sandbox_required: true,
        });
        let req = EvaluationRequest::new("sandboxed_tool", "execute", 1);
        let eval = gate.evaluate(&req);
        assert!(!eval.permitted);
    }
}
