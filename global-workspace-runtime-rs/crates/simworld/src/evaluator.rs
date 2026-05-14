//! EvaluatorRun: orchestrates N cycles of simulation using RuntimeLoop.
//!
//! The evaluator feeds scenario observations into RuntimeLoop and receives
//! RuntimeStepResult per cycle. Events from RuntimeStepResult.events are
//! appended to the evaluator's shared EventLog (no clone() disconnect).
//!
//! The scenario's `expected_action` is used ONLY for scoring `action_match_rate`.

use evidence::EvidenceSource;
use governed_memory::codex_adapter::evidence_entry_to_candidate;
use governed_memory::{
    MemoryAdmissionGate, RetrievalIntentCategory, RetrievalPlanner, RetrievalQuery, RetrievalRouter,
};
use memory::claim_store::ClaimStore;
use modulation::pressure::OperationalPressureState;
use modulation::self_model::SelfModel;
use runtime_core::event::WorldOutcome;
use runtime_core::reasoning_audit::ReasoningAudit;
use runtime_core::{EventLog, KeywordMemoryProvider, RuntimeEvent, RuntimeLoop};
use tools::{ToolGate, ToolPolicy};

use chrono::Utc;

use crate::environment::CooperativeSupportWorld;
use crate::evaluator_trace::EvaluatorTrace;
use crate::scorecard::{Scorecard, ScorecardBuilder};
use crate::sim_types::SimAction;

pub struct EvaluatorRun {
    pub world: CooperativeSupportWorld,
    pub log: EventLog,
    pub traces: Vec<EvaluatorTrace>,
}

impl EvaluatorRun {
    pub fn new(seed: u64, log_path: Option<std::path::PathBuf>) -> Self {
        let log = match log_path {
            Some(p) => EventLog::with_path(p),
            None => EventLog::new(),
        };
        Self {
            world: CooperativeSupportWorld::new(seed),
            log,
            traces: Vec::new(),
        }
    }

    /// Run `cycles` simulation steps and return a Scorecard.
    ///
    /// RuntimeLoop generates actions independently. `expected_action`
    /// is used only for scoring.
    pub fn run(&mut self, cycles: u64) -> Scorecard {
        let mut builder = ScorecardBuilder::new();
        // Subsystem initialization
        let mut tool_gate = ToolGate::new();
        tool_gate.register_policy(ToolPolicy {
            tool_id: "default_tool".into(),
            allowed_actions: vec!["read".into(), "write".into()],
            max_consecutive: 10,
            requires_confirmation: false,
            sandbox_required: false,
        });
        let mut rt = RuntimeLoop::with_tool_gate(Box::new(KeywordMemoryProvider::new()), tool_gate);
        let mut self_model = SelfModel::new();
        let mut evidence_vault = evidence::EvidenceVault::new();
        let mut claim_store = ClaimStore::new();
        let mut contradiction_engine = contradiction::ContradictionEngine::new();
        let mut pressure = OperationalPressureState::new();
        let admission_gate = MemoryAdmissionGate::default_policy();

        for cycle_id in 0..cycles {
            let scenario = self.world.next_scenario();
            let observation = scenario.name;
            let world_resources = self.world.resources;
            let expected_action: SimAction = scenario.expected_action.clone();
            let resource_before = self.world.resources;

            // ── RuntimeLoop pipeline ──────────────────────────────────
            // Retrieve bounded claims for this observation before scoring.
            let retrieval = claim_store.retrieve_for_observation(observation);
            let retrieval_query = RetrievalQuery {
                query_id: format!("retrieve_{cycle_id}"),
                query_text: observation.to_string(),
                context: Some("simworld_live_runtime".to_string()),
                intent_category: RetrievalIntentCategory::MemoryLookup,
                requires_verification: true,
                max_candidates: 10,
                confidence_threshold: 0.6,
                created_at: Utc::now(),
            };
            let retrieval_decision = RetrievalRouter::route(&retrieval_query);
            let _retrieval_plan = RetrievalPlanner::plan(&retrieval_query);
            let _ = self
                .log
                .append(RuntimeEvent::GovernedMemoryRetrievalPlanned {
                    cycle_id,
                    query_id: retrieval_query.query_id.clone(),
                    intent_category: format!("{:?}", retrieval_decision.intent),
                    recommended_action: retrieval_decision.recommended_action.clone(),
                    reason_codes: retrieval_decision
                        .reason_codes
                        .iter()
                        .map(|c| c.code.clone())
                        .collect(),
                });
            rt.has_evidence_backed_claims = retrieval
                .matched_claims
                .iter()
                .any(|c| c.evidence_id.is_some());
            rt.has_any_active_claims = !retrieval.matched_claims.is_empty();
            rt.has_disputed_claims = !retrieval.disputed_claims.is_empty();

            for claim_ref in retrieval
                .matched_claims
                .iter()
                .chain(retrieval.disputed_claims.iter())
            {
                let confidence = claim_store
                    .get(&claim_ref.claim_id)
                    .map(|c| c.confidence)
                    .unwrap_or(0.0);
                let _ = self.log.append(RuntimeEvent::ClaimRetrieved {
                    cycle_id,
                    claim_id: claim_ref.claim_id.clone(),
                    evidence_id: claim_ref.evidence_id.clone(),
                    status: format!("{:?}", claim_ref.status).to_lowercase(),
                    confidence,
                });
            }

            // Compute pressure and bias BEFORE cycle
            pressure.decay(0.1);
            let old_safety = pressure.safety_pressure;
            let old_uncertainty = pressure.uncertainty_pressure;

            pressure.safety_pressure =
                (pressure.safety_pressure + rt.internal_state.threat * 0.5).clamp(0.0, 1.0);
            pressure.uncertainty_pressure = (pressure.uncertainty_pressure
                + rt.internal_state.uncertainty * 0.5)
                .clamp(0.0, 1.0);
            let rp = if self.world.resources < 0.5 {
                0.7
            } else {
                1.0 - self.world.resources
            };
            pressure.resource_pressure = rp.clamp(0.0, 1.0);
            if !contradiction_engine.active().is_empty() {
                pressure.contradiction_pressure = 0.6;
            }

            // Build bias from pressure
            let bias = pressure.to_policy_bias();
            let runtime_bias = runtime_core::types::PressureBias {
                answer: bias.answer,
                ask_clarification: bias.ask_clarification,
                retrieve_memory: bias.retrieve_memory,
                refuse_unsafe: bias.refuse_unsafe,
                defer_insufficient_evidence: bias.defer_insufficient_evidence,
                summarize: bias.summarize,
                plan: bias.plan,
                execute_bounded_tool: bias.execute_bounded_tool,
                no_op: bias.no_op,
                internal_diagnostic: bias.internal_diagnostic,
            };

            // Run cycle with pressure bias applied
            let step =
                rt.run_cycle_with_bias(observation, cycle_id, world_resources, &Some(runtime_bias));
            let action_type = step.selected_action.clone();
            let sim_action: SimAction = action_type.clone().into();

            // ── Append Runtime events to shared log ─────────────────
            for event in &step.events {
                let _ = self.log.append(event.clone());
            }

            // ── World update event ───────────────────────────────────
            let outcome = self.world.apply_action(&sim_action, scenario);
            let is_unsafe = !sim_action.is_safe_for_users();

            let _ = self.log.append(RuntimeEvent::WorldStateUpdated {
                cycle_id,
                outcome: WorldOutcome {
                    resource_delta: outcome.resource_delta,
                    social_score: outcome.social_score,
                    harm_score: outcome.harm_score,
                    truth_score: outcome.truth_score,
                    kindness_score: outcome.kindness_score,
                    logic_score: outcome.logic_score,
                    utility_score: outcome.utility_score,
                    matches_expected: outcome.matches_expected,
                },
            });

            // ── Subsystem integration ─────────────────────────────────
            // Self-model: record action and resources
            self_model.record_action(action_type.to_string());
            self_model.set_resources(self.world.resources);

            // Evidence vault: store observation as evidence
            let ev_id = format!("ev_{cycle_id}");
            let ev_id_for_audit = ev_id.clone();
            let ev_idx = evidence_vault
                .append(
                    &ev_id,
                    EvidenceSource::Observation,
                    serde_json::json!({"observation": observation, "kind": "simworld"}),
                    0.7,
                )
                .unwrap_or(0);
            let ev_hash = evidence_vault
                .get(ev_idx)
                .map(|e| e.content_hash.clone())
                .unwrap_or_else(|| "unknown".into());
            let _ = self.log.append(RuntimeEvent::EvidenceStored {
                cycle_id,
                entry_id: ev_id,
                source: "observation".into(),
                confidence: 0.7,
                content_hash: ev_hash,
            });

            // Claim store: bounded evidence -> claim creation only.
            if let Some(entry) = evidence_vault.get(ev_idx).cloned() {
                let structured_entry = evidence::EvidenceEntry {
                    content: serde_json::json!({
                        // Keep the claim bounded and retrievable by using the
                        // observation text itself as the subject.
                        "subject": observation,
                        "predicate": format!("cycle_{cycle_id}_completed")
                    }),
                    ..entry
                };
                let candidate = evidence_entry_to_candidate(
                    &structured_entry.id,
                    observation,
                    &format!("cycle_{cycle_id}_completed"),
                    None,
                    &structured_entry.content_hash,
                );
                let admission = admission_gate.admit(&candidate);
                let claim_written =
                    admission.admitted && admission.storage_location == "active_claim";
                let _ = self
                    .log
                    .append(RuntimeEvent::GovernedMemoryAdmissionEvaluated {
                        cycle_id,
                        candidate_id: candidate.id.clone(),
                        decision_kind: admission.storage_location.clone(),
                        reason_codes: admission
                            .reason_codes
                            .iter()
                            .map(|c| c.code.clone())
                            .collect(),
                        confidence: admission.confidence,
                        source_trust_score: candidate.confidence,
                        live_hook: true,
                        claimstore_writer: "codex".to_string(),
                        governed_memory_writer: false,
                        claim_written,
                        override_applied: false,
                    });
                if claim_written {
                    if let Ok(claim) = claim_store.assert_from_evidence(&structured_entry) {
                        let claim_id = claim.id.clone();
                        let claim_subject = claim.subject.clone();
                        let claim_predicate = claim.predicate.clone();
                        let _ = self.log.append(RuntimeEvent::ClaimAsserted {
                            cycle_id,
                            claim_id: claim_id.clone(),
                            subject: claim_subject,
                            predicate: claim_predicate,
                        });
                        let _ = claim_store.validate(&claim_id);
                        let _ = self
                            .log
                            .append(RuntimeEvent::ClaimValidated { cycle_id, claim_id });
                    }
                }
            }

            let post_retrieval = claim_store.retrieve_for_observation(observation);
            for claim_ref in post_retrieval
                .matched_claims
                .iter()
                .chain(post_retrieval.disputed_claims.iter())
            {
                let confidence = claim_store
                    .get(&claim_ref.claim_id)
                    .map(|c| c.confidence)
                    .unwrap_or(0.0);
                let _ = self.log.append(RuntimeEvent::ClaimRetrieved {
                    cycle_id,
                    claim_id: claim_ref.claim_id.clone(),
                    evidence_id: claim_ref.evidence_id.clone(),
                    status: format!("{:?}", claim_ref.status).to_lowercase(),
                    confidence,
                });
            }

            // Contradiction detection: every 10 cycles
            if cycle_id % 10 == 0 && cycle_id > 0 {
                let checked_claim_ids = claim_store
                    .active_claims()
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<Vec<_>>();
                let contra_ids = contradiction_engine.detect(&claim_store);
                for cid in &contra_ids {
                    if let Some(c) = contradiction_engine.get(cid) {
                        let _ = self.log.append(RuntimeEvent::ContradictionDetected {
                            cycle_id,
                            claim_a: c.claim_a.clone(),
                            claim_b: c.claim_b.clone(),
                            subject: c.subject.clone(),
                        });
                    }
                }
                let _ = self.log.append(RuntimeEvent::ContradictionChecked {
                    cycle_id,
                    checked_claim_ids,
                    contradiction_ids: contra_ids,
                    active_contradictions: contradiction_engine.active().len(),
                });
            }

            // ── Pressure events (per-field, real old values) ────────
            if (old_safety - pressure.safety_pressure).abs() > 0.001 {
                let _ = self.log.append(RuntimeEvent::PressureUpdated {
                    cycle_id,
                    field: "safety".into(),
                    old_value: old_safety,
                    new_value: pressure.safety_pressure,
                    source: "Observation".into(),
                    reason: "threat from observation".into(),
                });
            }
            if (old_uncertainty - pressure.uncertainty_pressure).abs() > 0.001 {
                let _ = self.log.append(RuntimeEvent::PressureUpdated {
                    cycle_id,
                    field: "uncertainty".into(),
                    old_value: old_uncertainty,
                    new_value: pressure.uncertainty_pressure,
                    source: "Observation".into(),
                    reason: "uncertainty from observation".into(),
                });
            }
            let dominant: Vec<String> = pressure
                .dominant_pressures(3)
                .iter()
                .map(|f| f.as_str().to_string())
                .collect();
            let _ = self.log.append(RuntimeEvent::PolicyBiasApplied {
                cycle_id,
                dominant_pressures: dominant.clone(),
                selected_action: action_type.to_string(),
            });

            let retrieval_for_audit = claim_store.retrieve_for_observation(observation);
            let claim_ids_for_audit = retrieval_for_audit
                .matched_claims
                .iter()
                .map(|c| c.claim_id.clone())
                .collect::<Vec<_>>();
            let disputed_claim_ids_for_audit = retrieval_for_audit
                .disputed_claims
                .iter()
                .map(|c| c.claim_id.clone())
                .collect::<Vec<_>>();
            let contradiction_ids_for_audit = contradiction_engine
                .active()
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>();

            // Reasoning audit: generate per cycle with evidence/claim/pressure refs
            let audit = ReasoningAudit::new(
                cycle_id,
                observation,
                action_type.clone(),
                format!(
                    "Evaluator cycle {cycle_id}: selected {} backed by evidence {}",
                    action_type, ev_id_for_audit
                ),
            )
            .with_symbols(
                step.symbolic_activations
                    .iter()
                    .map(|s| s.symbol_id.clone())
                    .collect(),
            )
            .with_memory_hits(step.memory_hits.iter().map(|h| h.key.clone()).collect())
            .with_evidence(vec![ev_id_for_audit])
            .with_claim_ids(claim_ids_for_audit.clone())
            .with_disputed_claim_ids(disputed_claim_ids_for_audit)
            .with_contradiction_ids(contradiction_ids_for_audit.clone())
            .with_dominant_pressures(dominant.clone());
            let _ = self.log.append(RuntimeEvent::ReasoningAuditGenerated {
                cycle_id,
                audit_id: audit.audit_id.clone(),
                selected_action: action_type.to_string(),
                evidence_ids: audit.evidence_ids.clone(),
                claim_ids: claim_ids_for_audit,
                contradiction_ids: contradiction_ids_for_audit,
                dominant_pressures: dominant.clone(),
                audit_text: audit.to_text(),
            });

            builder.record_outcome(
                outcome.total_score(),
                outcome.matches_expected,
                outcome.harm_score,
                outcome.truth_score,
                outcome.social_score,
                outcome.utility_score,
                is_unsafe,
                false,
            );

            // ── Populate evaluator trace from RuntimeStepResult ──────
            let mut trace = EvaluatorTrace::new(cycle_id, scenario.name, observation);
            trace.selected_action = action_type.to_string();
            trace.expected_action = expected_action.as_str().to_string();
            trace.action_match = sim_action == expected_action;
            trace.risk_score = rt.internal_state.threat;
            trace.uncertainty_score = rt.internal_state.uncertainty;
            trace.resource_score_before = resource_before;
            trace.resource_score_after = self.world.resources;
            trace.unsafe_action_flag = is_unsafe;
            trace.memory_hits_count = step.memory_hits.len();
            trace.memory_hit_ids = step.memory_hits.iter().map(|h| h.key.clone()).collect();
            trace.symbolic_context_count = step.symbolic_activations.len();
            trace.symbolic_symbol_ids = step
                .symbolic_activations
                .iter()
                .map(|s| s.symbol_id.clone())
                .collect();
            trace.candidate_actions = step
                .candidate_actions
                .iter()
                .map(|c| c.action_type.to_string())
                .collect();
            trace.rejected_actions = step
                .rejected_actions
                .iter()
                .map(|r| crate::evaluator_trace::RejectionRecord {
                    action: r.action_type.to_string(),
                    reason: r.reason.clone(),
                })
                .collect();
            trace.policy_scores = step
                .policy_scores
                .iter()
                .map(|p| crate::evaluator_trace::TracePolicyScore {
                    action_type: p.action_type.to_string(),
                    base_score: p.base_score,
                    bonus: p.bonus,
                    final_score: p.final_score,
                    modifiers: p.modifiers.clone(),
                })
                .collect();
            trace.runtime_events_count = step.events.len();
            trace.selection_reason = step.selection_reason.clone();
            if let Some(ref ctx) = rt.last_context {
                trace.observation_kind = ctx.kind.as_str().to_string();
            }

            self.traces.push(trace);

            // ── Provider counters report (Simulated, always zero) ────
            let _ = self.log.append(RuntimeEvent::ProviderCountersReported {
                cycle_id,
                snapshot: runtime_core::event::ProviderCountersSnapshot {
                    local_requests: 0,
                    local_successes: 0,
                    local_failures: 0,
                    local_disabled_blocks: 0,
                    cloud_requests: 0,
                    external_requests: 0,
                    feature_enabled: false,
                },
            });
        }

        builder.set_final_resources(self.world.resources);
        builder.build()
    }

    /// Return all traces as JSON.
    pub fn traces_as_json(&self) -> String {
        serde_json::to_string_pretty(&self.traces).unwrap_or_default()
    }

    /// Run evaluation using NL scenarios instead of label-like scenarios.
    /// Reuses the existing RuntimeLoop pipeline; NL observations are fed
    /// through ObservationInterpreter for keyword-based classification.
    pub fn run_nl(&mut self) -> Scorecard {
        let scenarios = crate::nl_scenarios::NLScenarioSet::curated_set();
        let all = scenarios.all_scenarios();
        let cycles = all.len() as u64;
        self.run_with_scenarios(&all, cycles)
    }

    /// Run with custom scenario texts.
    pub fn run_with_scenarios(
        &mut self,
        scenarios: &[&crate::nl_scenarios::NLScenario],
        cycles: u64,
    ) -> Scorecard {
        let mut builder = ScorecardBuilder::new();
        let mut tool_gate = ToolGate::new();
        tool_gate.register_policy(ToolPolicy {
            tool_id: "default_tool".into(),
            allowed_actions: vec!["read".into(), "write".into()],
            max_consecutive: 10,
            requires_confirmation: false,
            sandbox_required: false,
        });
        let mut rt = RuntimeLoop::with_tool_gate(Box::new(KeywordMemoryProvider::new()), tool_gate);
        let mut self_model = SelfModel::new();
        let mut evidence_vault = evidence::EvidenceVault::new();
        let mut claim_store = ClaimStore::new();
        let mut contradiction_engine = contradiction::ContradictionEngine::new();
        let mut pressure = OperationalPressureState::new();
        let admission_gate = MemoryAdmissionGate::default_policy();

        for cycle_id in 0..cycles {
            let idx = (cycle_id as usize) % scenarios.len();
            let scenario = scenarios[idx];
            let observation = scenario.text.as_str();
            let expected_str = scenario.expected_action.as_str();
            let resource_before = self.world.resources;

            // Retrieve bounded claims for this observation before scoring.
            let retrieval = claim_store.retrieve_for_observation(observation);
            let retrieval_query = RetrievalQuery {
                query_id: format!("nl_retrieve_{cycle_id}"),
                query_text: observation.to_string(),
                context: Some("simworld_nl_runtime".to_string()),
                intent_category: RetrievalIntentCategory::MemoryLookup,
                requires_verification: true,
                max_candidates: 10,
                confidence_threshold: 0.6,
                created_at: Utc::now(),
            };
            let retrieval_decision = RetrievalRouter::route(&retrieval_query);
            let _retrieval_plan = RetrievalPlanner::plan(&retrieval_query);
            let _ = self
                .log
                .append(RuntimeEvent::GovernedMemoryRetrievalPlanned {
                    cycle_id,
                    query_id: retrieval_query.query_id.clone(),
                    intent_category: format!("{:?}", retrieval_decision.intent),
                    recommended_action: retrieval_decision.recommended_action.clone(),
                    reason_codes: retrieval_decision
                        .reason_codes
                        .iter()
                        .map(|c| c.code.clone())
                        .collect(),
                });
            rt.has_evidence_backed_claims = retrieval
                .matched_claims
                .iter()
                .any(|c| c.evidence_id.is_some());
            rt.has_any_active_claims = !retrieval.matched_claims.is_empty();
            rt.has_disputed_claims = !retrieval.disputed_claims.is_empty();

            for claim_ref in retrieval
                .matched_claims
                .iter()
                .chain(retrieval.disputed_claims.iter())
            {
                let confidence = claim_store
                    .get(&claim_ref.claim_id)
                    .map(|c| c.confidence)
                    .unwrap_or(0.0);
                let _ = self.log.append(RuntimeEvent::ClaimRetrieved {
                    cycle_id,
                    claim_id: claim_ref.claim_id.clone(),
                    evidence_id: claim_ref.evidence_id.clone(),
                    status: format!("{:?}", claim_ref.status).to_lowercase(),
                    confidence,
                });
            }

            // Compute pressure and bias (same as standard run)
            pressure.decay(0.1);
            let old_safety = pressure.safety_pressure;
            let old_uncertainty = pressure.uncertainty_pressure;
            let old_resource = pressure.resource_pressure;
            pressure.safety_pressure =
                (pressure.safety_pressure + rt.internal_state.threat * 0.5).clamp(0.0, 1.0);
            pressure.uncertainty_pressure = (pressure.uncertainty_pressure
                + rt.internal_state.uncertainty * 0.5)
                .clamp(0.0, 1.0);
            let rp = if self.world.resources < 0.5 {
                0.7
            } else {
                1.0 - self.world.resources
            };
            pressure.resource_pressure = rp.clamp(0.0, 1.0);
            if !contradiction_engine.active().is_empty() {
                pressure.contradiction_pressure = 0.6;
            }

            let bias = pressure.to_policy_bias();
            let runtime_bias = runtime_core::types::PressureBias {
                answer: bias.answer,
                ask_clarification: bias.ask_clarification,
                retrieve_memory: bias.retrieve_memory,
                refuse_unsafe: bias.refuse_unsafe,
                defer_insufficient_evidence: bias.defer_insufficient_evidence,
                summarize: bias.summarize,
                plan: bias.plan,
                execute_bounded_tool: bias.execute_bounded_tool,
                no_op: bias.no_op,
                internal_diagnostic: bias.internal_diagnostic,
            };

            let step = rt.run_cycle_with_bias(
                observation,
                cycle_id,
                self.world.resources,
                &Some(runtime_bias),
            );
            let action_type = step.selected_action.clone();
            let sim_action: SimAction = action_type.clone().into();

            for event in &step.events {
                let _ = self.log.append(event.clone());
            }

            // Use world's own scenario for resource model, NL text for observation
            let world_scenario = self.world.next_scenario();
            let outcome = self.world.apply_action(&sim_action, world_scenario);
            let is_unsafe = !sim_action.is_safe_for_users();

            let _ = self.log.append(RuntimeEvent::WorldStateUpdated {
                cycle_id,
                outcome: WorldOutcome {
                    resource_delta: outcome.resource_delta,
                    social_score: outcome.social_score,
                    harm_score: outcome.harm_score,
                    truth_score: outcome.truth_score,
                    kindness_score: outcome.kindness_score,
                    logic_score: outcome.logic_score,
                    utility_score: outcome.utility_score,
                    matches_expected: sim_action.as_str() == expected_str,
                },
            });

            // Subsystem integration (same as standard run)
            self_model.record_action(action_type.to_string());
            self_model.set_resources(self.world.resources);

            let ev_id = format!("nl_ev_{cycle_id}");
            let ev_id_for_audit = ev_id.clone();
            let ev_idx = evidence_vault
                .append(
                    &ev_id,
                    EvidenceSource::Observation,
                    serde_json::json!({"text": observation, "expected": expected_str}),
                    0.7,
                )
                .unwrap_or(0);
            let ev_hash = evidence_vault
                .get(ev_idx)
                .map(|e| e.content_hash.clone())
                .unwrap_or_else(|| "unknown".into());
            let _ = self.log.append(RuntimeEvent::EvidenceStored {
                cycle_id,
                entry_id: ev_id,
                source: "nl_observation".into(),
                confidence: 0.7,
                content_hash: ev_hash,
            });

            if let Some(entry) = evidence_vault.get(ev_idx).cloned() {
                let structured_entry = evidence::EvidenceEntry {
                    content: serde_json::json!({
                        // Keep the claim bounded and retrievable by using the
                        // observation text itself as the subject.
                        "subject": observation,
                        "predicate": format!("cycle_{cycle_id}")
                    }),
                    ..entry
                };
                let candidate = evidence_entry_to_candidate(
                    &structured_entry.id,
                    observation,
                    &format!("cycle_{cycle_id}"),
                    None,
                    &structured_entry.content_hash,
                );
                let admission = admission_gate.admit(&candidate);
                let claim_written =
                    admission.admitted && admission.storage_location == "active_claim";
                let _ = self
                    .log
                    .append(RuntimeEvent::GovernedMemoryAdmissionEvaluated {
                        cycle_id,
                        candidate_id: candidate.id.clone(),
                        decision_kind: admission.storage_location.clone(),
                        reason_codes: admission
                            .reason_codes
                            .iter()
                            .map(|c| c.code.clone())
                            .collect(),
                        confidence: admission.confidence,
                        source_trust_score: candidate.confidence,
                        live_hook: true,
                        claimstore_writer: "codex".to_string(),
                        governed_memory_writer: false,
                        claim_written,
                        override_applied: false,
                    });
                if claim_written {
                    if let Ok(claim) = claim_store.assert_from_evidence(&structured_entry) {
                        let claim_id = claim.id.clone();
                        let claim_subject = claim.subject.clone();
                        let claim_predicate = claim.predicate.clone();
                        let _ = self.log.append(RuntimeEvent::ClaimAsserted {
                            cycle_id,
                            claim_id: claim_id.clone(),
                            subject: claim_subject,
                            predicate: claim_predicate,
                        });
                        let _ = claim_store.validate(&claim_id);
                        let _ = self
                            .log
                            .append(RuntimeEvent::ClaimValidated { cycle_id, claim_id });
                    }
                }
            }

            let post_retrieval = claim_store.retrieve_for_observation(observation);
            for claim_ref in post_retrieval
                .matched_claims
                .iter()
                .chain(post_retrieval.disputed_claims.iter())
            {
                let confidence = claim_store
                    .get(&claim_ref.claim_id)
                    .map(|c| c.confidence)
                    .unwrap_or(0.0);
                let _ = self.log.append(RuntimeEvent::ClaimRetrieved {
                    cycle_id,
                    claim_id: claim_ref.claim_id.clone(),
                    evidence_id: claim_ref.evidence_id.clone(),
                    status: format!("{:?}", claim_ref.status).to_lowercase(),
                    confidence,
                });
            }

            if cycle_id % 5 == 0 && cycle_id > 0 {
                let checked_claim_ids = claim_store
                    .active_claims()
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<Vec<_>>();
                let contra_ids = contradiction_engine.detect(&claim_store);
                for cid in &contra_ids {
                    if let Some(c) = contradiction_engine.get(cid) {
                        let _ = self.log.append(RuntimeEvent::ContradictionDetected {
                            cycle_id,
                            claim_a: c.claim_a.clone(),
                            claim_b: c.claim_b.clone(),
                            subject: c.subject.clone(),
                        });
                    }
                }
                let _ = self.log.append(RuntimeEvent::ContradictionChecked {
                    cycle_id,
                    checked_claim_ids,
                    contradiction_ids: contra_ids,
                    active_contradictions: contradiction_engine.active().len(),
                });
            }

            // Pressure events (per-field, real old values)
            if (old_safety - pressure.safety_pressure).abs() > 0.001 {
                let _ = self.log.append(RuntimeEvent::PressureUpdated {
                    cycle_id,
                    field: "safety".into(),
                    old_value: old_safety,
                    new_value: pressure.safety_pressure,
                    source: "Observation".into(),
                    reason: "threat from NL observation".into(),
                });
            }
            if (old_uncertainty - pressure.uncertainty_pressure).abs() > 0.001 {
                let _ = self.log.append(RuntimeEvent::PressureUpdated {
                    cycle_id,
                    field: "uncertainty".into(),
                    old_value: old_uncertainty,
                    new_value: pressure.uncertainty_pressure,
                    source: "Observation".into(),
                    reason: "uncertainty from NL observation".into(),
                });
            }
            if (old_resource - pressure.resource_pressure).abs() > 0.001 {
                let _ = self.log.append(RuntimeEvent::PressureUpdated {
                    cycle_id,
                    field: "resource".into(),
                    old_value: old_resource,
                    new_value: pressure.resource_pressure,
                    source: "ResourceState".into(),
                    reason: "resource level change in NL mode".into(),
                });
            }
            let dominant: Vec<String> = pressure
                .dominant_pressures(3)
                .iter()
                .map(|f| f.as_str().to_string())
                .collect();
            let _ = self.log.append(RuntimeEvent::PolicyBiasApplied {
                cycle_id,
                dominant_pressures: dominant.clone(),
                selected_action: action_type.to_string(),
            });

            let retrieval_for_audit = claim_store.retrieve_for_observation(observation);
            let claim_ids_for_audit = retrieval_for_audit
                .matched_claims
                .iter()
                .map(|c| c.claim_id.clone())
                .collect::<Vec<_>>();
            let contradiction_ids_for_audit = contradiction_engine
                .active()
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>();

            // Reasoning audit: NL path with evidence/claim/pressure refs
            let audit = ReasoningAudit::new(
                cycle_id,
                observation,
                action_type.clone(),
                format!(
                    "NL cycle {cycle_id}: selected {} backed by evidence {}",
                    action_type, ev_id_for_audit
                ),
            )
            .with_evidence(vec![ev_id_for_audit])
            .with_claim_ids(claim_ids_for_audit.clone())
            .with_contradiction_ids(contradiction_ids_for_audit.clone())
            .with_dominant_pressures(dominant.clone());
            let _ = self.log.append(RuntimeEvent::ReasoningAuditGenerated {
                cycle_id,
                audit_id: audit.audit_id.clone(),
                selected_action: action_type.to_string(),
                evidence_ids: audit.evidence_ids.clone(),
                claim_ids: claim_ids_for_audit,
                contradiction_ids: contradiction_ids_for_audit,
                dominant_pressures: dominant,
                audit_text: audit.to_text(),
            });

            let action_match = action_type.to_string() == expected_str;
            builder.record_outcome(
                outcome.total_score(),
                action_match,
                outcome.harm_score,
                outcome.truth_score,
                outcome.social_score,
                outcome.utility_score,
                is_unsafe,
                false,
            );

            let mut trace = EvaluatorTrace::new(cycle_id, &scenario.id, observation);
            trace.selected_action = action_type.to_string();
            trace.expected_action = expected_str.to_string();
            trace.action_match = action_match;
            trace.risk_score = rt.internal_state.threat;
            trace.uncertainty_score = rt.internal_state.uncertainty;
            trace.resource_score_before = resource_before;
            trace.resource_score_after = self.world.resources;
            trace.unsafe_action_flag = is_unsafe;
            trace.selection_reason = step.selection_reason.clone();
            self.traces.push(trace);

            // ── Provider counters report (Simulated, always zero) ────
            let _ = self.log.append(RuntimeEvent::ProviderCountersReported {
                cycle_id,
                snapshot: runtime_core::event::ProviderCountersSnapshot {
                    local_requests: 0,
                    local_successes: 0,
                    local_failures: 0,
                    local_disabled_blocks: 0,
                    cloud_requests: 0,
                    external_requests: 0,
                    feature_enabled: false,
                },
            });
        }

        builder.set_final_resources(self.world.resources);
        builder.build()
    }
}
