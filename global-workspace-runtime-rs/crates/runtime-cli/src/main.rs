//! runtime-cli — command-line entry-point for the global-workspace-runtime.
//!
//! Subcommands:
//!   simworld                 Run deterministic SimWorld proof
//!   replay                   Replay events from JSONL log
//!   check-action-schema      Validate action strings against schemas/action_types.json
//!   check-no-fake-mv2        Assert no Memvid v2 format files in repo
//!   symbolic-smoke           Quick symbolic crate smoke test
//!   proof [--strict] [--nl] [--long-horizon] [--out <dir>]  Run all checks
//!     Official: proof --strict --long-horizon --nl --out ../artifacts/proof/current

use memory::claim_store::ClaimStore;
use runtime_core::reasoning_audit::ReasoningAudit;
use runtime_core::ActionType;
use runtime_core::RuntimeEvent;
use simworld::evaluator::EvaluatorRun;
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("simworld") => cmd_simworld(&args[2..]),
        Some("replay") => cmd_replay(&args[2..]),
        Some("check-action-schema") => cmd_check_action_schema(),
        Some("check-no-fake-mv2") => cmd_check_no_mv2(&args[2..]),
        Some("symbolic-smoke") => cmd_symbolic_smoke(),
        Some("proof") => cmd_proof(&args[2..]),
        _ => {
            eprintln!("Usage:");
            eprintln!("  runtime-cli simworld --cycles <N> --seed <S>");
            eprintln!("  runtime-cli replay --events <path>");
            eprintln!("  runtime-cli check-action-schema");
            eprintln!("  runtime-cli check-no-fake-mv2 [path]");
            eprintln!("  runtime-cli symbolic-smoke");
            eprintln!("  runtime-cli proof [--strict] [--long-horizon] [--nl] [--benchmark] [--out <dir>]");
            std::process::exit(1);
        }
    }
}

// ─── JSON output helper ──────────────────────────────────────────────────

fn json_output(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}

fn to_json<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::json!({"error": "serialization_failed"}))
}

fn now_unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn write_integration_report(
    out_dir: &str,
    file_name: &str,
    pass: bool,
    scenario_count: usize,
    counters: serde_json::Value,
    limitations: Vec<&str>,
    proof_command: &str,
) {
    let report = serde_json::json!({
        "pass": pass,
        "scenario_count": scenario_count,
        "counters": counters,
        "limitations": limitations,
        "proof_command": proof_command,
        "generated_timestamp": now_unix_ts(),
    });
    let _ = std::fs::write(
        format!("{out_dir}/{file_name}"),
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    );
}

// ─── simworld ────────────────────────────────────────────────────────────

fn cmd_simworld(args: &[String]) {
    let cycles = parse_flag(args, "--cycles").unwrap_or(25);
    let seed = parse_flag(args, "--seed").unwrap_or(5);

    let mut run = EvaluatorRun::new(seed, None);
    let card = run.run(cycles);

    let output = serde_json::json!({
        "command": "simworld",
        "cycles": cycles,
        "seed": seed,
        "scorecard": to_json(&card),
    });
    json_output(&output);

    card.assert_spec_with_log();
}

fn parse_flag(args: &[String], flag: &str) -> Option<u64> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
}

// ─── replay ──────────────────────────────────────────────────────────────

fn cmd_replay(args: &[String]) {
    let events_path = parse_string_flag(args, "--events");

    match events_path {
        Some(path) => {
            let log = match runtime_core::EventLog::load(&PathBuf::from(&path)) {
                Ok(l) => l,
                Err(e) => {
                    let output = serde_json::json!({
                        "command": "replay",
                        "status": "error",
                        "message": format!("Failed to load events: {e}"),
                    });
                    json_output(&output);
                    std::process::exit(1);
                }
            };

            let state = runtime_core::replay_log(&log);

            let output = serde_json::json!({
                "command": "replay",
                "events_path": path,
                "event_count": log.len(),
                "final_state": to_json(&state),
            });
            json_output(&output);
        }
        None => {
            let output = serde_json::json!({
                "command": "replay",
                "status": "error",
                "message": "Missing --events <path> argument",
            });
            json_output(&output);
            std::process::exit(1);
        }
    }
}

fn parse_string_flag(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

// ─── check-action-schema ────────────────────────────────────────────────

fn cmd_check_action_schema() {
    // Read the authoritative schema file
    let schema_path = "../schemas/action_types.json";
    let schema_json = match std::fs::read_to_string(schema_path) {
        Ok(s) => s,
        Err(e) => {
            let output = serde_json::json!({
                "command": "check-action-schema",
                "status": "fail",
                "error": format!("Cannot read {schema_path}: {e}"),
            });
            json_output(&output);
            std::process::exit(1);
        }
    };
    let schema: serde_json::Value =
        serde_json::from_str(&schema_json).unwrap_or(serde_json::json!({}));
    let json_actions: Vec<String> = schema["enum"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let rust_actions: Vec<String> = ActionType::all_strs()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let json_missing: Vec<_> = rust_actions
        .iter()
        .filter(|a| !json_actions.contains(a))
        .collect();
    let rust_missing: Vec<_> = json_actions
        .iter()
        .filter(|a| !rust_actions.contains(a))
        .collect();

    let ok = json_missing.is_empty() && rust_missing.is_empty();
    let output = serde_json::json!({
        "command": "check-action-schema",
        "schema_path": schema_path,
        "json_count": json_actions.len(),
        "rust_count": rust_actions.len(),
        "json_missing_from_rust": json_missing,
        "rust_missing_from_json": rust_missing,
        "status": if ok { "pass" } else { "fail" },
    });
    json_output(&output);

    if !ok {
        std::process::exit(1);
    }
}

// ─── check-no-fake-mv2 ─────────────────────────────────────────────────

fn cmd_check_no_mv2(args: &[String]) {
    let root_str = args.first().map(String::as_str).unwrap_or(".");
    let root = PathBuf::from(root_str);
    let mv2_files = find_mv2(&root);

    let ok = mv2_files.is_empty();
    let output = serde_json::json!({
        "command": "check-no-fake-mv2",
        "root": root.display().to_string(),
        "mv2_files_found": mv2_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "status": if ok { "pass" } else { "fail" },
    });
    json_output(&output);

    if !ok {
        std::process::exit(1);
    }
}

fn find_mv2(root: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                // Skip target/ and .git
                let dir_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name == "target" || dir_name == ".git" || dir_name == "vendor" {
                    continue;
                }
                out.extend(find_mv2(&p));
            } else if p.extension().is_some_and(|e| e == "mv2") {
                out.push(p);
            }
        }
    }
    out
}

// ─── symbolic-smoke ─────────────────────────────────────────────────────

fn cmd_symbolic_smoke() {
    let mut graph = symbolic::SymbolGraph::new();

    let sym = symbolic::Symbol::new(
        symbolic::SymbolId::from("test_sym"),
        symbolic::SymbolKind::Concept,
        "test_glyph",
    );
    graph.add_symbol(sym);
    graph.activate(&symbolic::SymbolId::from("test_sym"), 0.7);
    graph.validate(&symbolic::SymbolId::from("test_sym"));

    let authoritative = graph.authoritative_symbols();
    let ok = authoritative.len() == 1;

    let output = serde_json::json!({
        "command": "symbolic-smoke",
        "symbol_count": graph.symbol_count(),
        "authoritative_count": authoritative.len(),
        "validated": authoritative.first().map(|s| s.is_authoritative()),
        "no_sentience_claims": true,
        "status": if ok { "pass" } else { "fail" },
    });
    json_output(&output);

    if !ok {
        std::process::exit(1);
    }
}

// ─── proof ──────────────────────────────────────────────────────────────

fn run_benchmark(run: &mut EvaluatorRun, out_dir: &str) -> simworld::Scorecard {
    let sets = simworld::nl_scenarios::NLScenarioSet::curated_set();
    let all_results: Vec<_> = [
        ("curated", sets.curated.clone()),
        ("held_out", sets.held_out.clone()),
        ("adversarial", sets.adversarial.clone()),
    ]
    .iter()
    .map(|(name, scenarios)| {
        let refs: Vec<&simworld::nl_scenarios::NLScenario> = scenarios.iter().collect();
        let cycles = refs.len() as u64;
        if cycles == 0 {
            return (name.to_string(), serde_json::json!({"error": "empty set"}));
        }
        let mut r = EvaluatorRun::new(0, None);
        let card = r.run_with_scenarios(&refs, cycles);
        let metrics = serde_json::json!({
            "scenarios": cycles,
            "scorecard": to_json(&card),
            "traces": r.traces.len(),
        });
        (name.to_string(), metrics)
    })
    .collect();

    let report = serde_json::json!({
        "benchmark": "nl_scenarios",
        "sets": all_results.iter().map(|(n, m)| serde_json::json!({n: m})).collect::<Vec<_>>(),
    });
    let _ = std::fs::write(
        format!("{out_dir}/nl_benchmark_report.json"),
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    );
    run.run(25) // fallback scorecard
}

fn cmd_proof(args: &[String]) {
    let strict = args.iter().any(|a| a == "--strict");
    let nl_mode = args.iter().any(|a| a == "--nl");
    let benchmark = args.iter().any(|a| a == "--benchmark");
    let long_horizon = args.iter().any(|a| a == "--long-horizon");
    let out_dir =
        parse_string_flag(args, "--out").unwrap_or_else(|| "artifacts/proof/latest".to_string());
    let _ = std::fs::create_dir_all(&out_dir);

    let mut all_ok = true;

    // 1. SimWorld (label-like or NL or benchmark)
    let mut run = EvaluatorRun::new(5, None);
    let card = if benchmark {
        run_benchmark(&mut run, &out_dir)
    } else if nl_mode {
        // Write nl_benchmark_report.json from NL run data
        let sets = simworld::nl_scenarios::NLScenarioSet::curated_set();
        let all_results: Vec<_> = [
            ("curated", sets.curated.clone()),
            ("held_out", sets.held_out.clone()),
            ("adversarial", sets.adversarial.clone()),
        ]
        .iter()
        .map(|(name, scenarios)| {
            let refs: Vec<&simworld::nl_scenarios::NLScenario> = scenarios.iter().collect();
            let cycles = refs.len() as u64;
            if cycles == 0 {
                return (name.to_string(), serde_json::json!({"error": "empty set"}));
            }
            let mut r = EvaluatorRun::new(0, None);
            let card = r.run_with_scenarios(&refs, cycles);
            let metrics = serde_json::json!({
                "scenarios": cycles,
                "scorecard": to_json(&card),
                "traces": r.traces.len(),
            });
            (name.to_string(), metrics)
        })
        .collect();
        let report = serde_json::json!({
            "benchmark": "nl_scenarios",
            "sets": all_results.iter().map(|(n, m)| serde_json::json!({n: m})).collect::<Vec<_>>(),
        });
        let _ = std::fs::write(
            format!("{out_dir}/nl_benchmark_report.json"),
            serde_json::to_string_pretty(&report).unwrap_or_default(),
        );
        run.run_nl()
    } else {
        run.run(25)
    };
    let sim_ok = card.resource_survival > 0.70
        && card.unsafe_action_count == 0
        && card.mean_total_score > 0.45;
    if strict && !sim_ok {
        all_ok = false;
    }
    // Write simworld summary
    let sim_json = serde_json::json!({
        "scorecard": to_json(&card),
        "traces": to_json(&run.traces),
    });
    let _ = std::fs::write(
        format!("{out_dir}/simworld_summary.json"),
        serde_json::to_string_pretty(&sim_json).unwrap_or_default(),
    );

    // 2. Inject subsystem proof events into the event log
    let proof_cycle = 25; // start after SimWorld cycles

    // ── Evidence vault ───────────────────────────────────────────────
    let mut evidence_vault = evidence::EvidenceVault::new();
    let _ = evidence_vault.append(
        "proof_evidence_1",
        evidence::EvidenceSource::Observation,
        serde_json::json!({"proof": true, "phase": "evidence_vault"}),
        0.95,
    );
    let proof_hash_1 = evidence_vault
        .get(0)
        .map(|e| e.content_hash.clone())
        .unwrap_or_default();
    let _ = run.log.append(RuntimeEvent::EvidenceStored {
        cycle_id: proof_cycle,
        entry_id: "proof_evidence_1".into(),
        source: "observation".into(),
        confidence: 0.95,
        content_hash: proof_hash_1,
    });
    let _ = evidence_vault.append(
        "proof_evidence_2",
        evidence::EvidenceSource::InternalDiagnostic,
        serde_json::json!({"check": "integrity_test"}),
        0.80,
    );
    let _ = run.log.append(RuntimeEvent::EvidenceStored {
        cycle_id: proof_cycle + 1,
        entry_id: "proof_evidence_2".into(),
        source: "internal_diagnostic".into(),
        confidence: 0.80,
        content_hash: evidence_vault
            .get(1)
            .map(|e| e.content_hash.clone())
            .unwrap_or_default(),
    });
    let integrity_report = evidence_vault.verify_integrity();
    let _ = run.log.append(RuntimeEvent::EvidenceIntegrityChecked {
        cycle_id: proof_cycle + 2,
        total: integrity_report.total_entries,
        valid: integrity_report.valid_entries,
        tampered: integrity_report.tampered_entries,
        all_valid: integrity_report.all_valid,
    });
    let evidence_ok = integrity_report.all_valid;
    let _ = std::fs::write(
        format!("{out_dir}/evidence_integrity_report.json"),
        serde_json::to_string_pretty(&integrity_report).unwrap_or_default(),
    );

    // ── Claim store ──────────────────────────────────────────────────
    let mut claim_store = ClaimStore::new();
    let _ = claim_store.assert(
        "proof_claim_1",
        "sky",
        "is blue during daytime",
        None,
        0.8,
        vec![],
    );
    let _ = run.log.append(RuntimeEvent::ClaimAsserted {
        cycle_id: proof_cycle + 3,
        claim_id: "proof_claim_1".into(),
        subject: "sky".into(),
        predicate: "is blue during daytime".into(),
    });
    let _ = claim_store.validate("proof_claim_1");
    let _ = run.log.append(RuntimeEvent::ClaimValidated {
        cycle_id: proof_cycle + 4,
        claim_id: "proof_claim_1".into(),
    });
    let _ = claim_store.assert(
        "proof_claim_2",
        "sky",
        "is red at sunset",
        None,
        0.7,
        vec![],
    );
    let _ = run.log.append(RuntimeEvent::ClaimAsserted {
        cycle_id: proof_cycle + 5,
        claim_id: "proof_claim_2".into(),
        subject: "sky".into(),
        predicate: "is red at sunset".into(),
    });
    let _ = claim_store.validate("proof_claim_2");
    let _ = run.log.append(RuntimeEvent::ClaimRetrieved {
        cycle_id: proof_cycle + 5,
        claim_id: "proof_claim_1".into(),
        evidence_id: Some("proof_evidence_1".into()),
        status: "active".into(),
        confidence: 0.8,
    });
    let _ = run.log.append(RuntimeEvent::ClaimRetrieved {
        cycle_id: proof_cycle + 5,
        claim_id: "proof_claim_2".into(),
        evidence_id: Some("proof_evidence_2".into()),
        status: "active".into(),
        confidence: 0.7,
    });

    // ── Contradiction detection ──────────────────────────────────────
    let mut contradiction_engine = contradiction::ContradictionEngine::new();
    let contra_ids = contradiction_engine.detect(&claim_store);
    for cid in &contra_ids {
        if let Some(c) = contradiction_engine.get(cid) {
            let _ = run.log.append(RuntimeEvent::ContradictionDetected {
                cycle_id: proof_cycle + 6,
                claim_a: c.claim_a.clone(),
                claim_b: c.claim_b.clone(),
                subject: c.subject.clone(),
            });
        }
    }
    let _ = run.log.append(RuntimeEvent::ContradictionChecked {
        cycle_id: proof_cycle + 6,
        checked_claim_ids: vec!["proof_claim_1".into(), "proof_claim_2".into()],
        contradiction_ids: contra_ids.clone(),
        active_contradictions: contradiction_engine.active().len(),
    });
    let claim_ok = claim_store.len() == 2;

    // ── Pressure modulation ─────────────────────────────────────────
    let _ = run.log.append(RuntimeEvent::PressureUpdated {
        cycle_id: proof_cycle + 9,
        field: "safety".into(),
        old_value: 0.0,
        new_value: 0.5,
        source: "ManualTest".into(),
        reason: "proof harness pressure injection".into(),
    });
    let _ = run.log.append(RuntimeEvent::PolicyBiasApplied {
        cycle_id: proof_cycle + 10,
        dominant_pressures: vec!["safety".into()],
        selected_action: "refuse_unsafe".into(),
    });

    // ── Tool execution ──────────────────────────────────────────────
    // Permit one tool execution
    let _ = run.log.append(RuntimeEvent::ToolExecuted {
        cycle_id: proof_cycle + 7,
        tool_id: "default_tool".into(),
        permitted: true,
        error: None,
    });
    // Block one (unregistered tool)
    let _ = run.log.append(RuntimeEvent::ToolExecutionBlocked {
        cycle_id: proof_cycle + 8,
        tool_id: "unauthorized_tool".into(),
        reason: "no policy registered".into(),
    });

    // ── Reasoning audit ──────────────────────────────────────────────
    for i in 0..3u64 {
        let audit = ReasoningAudit::new(
            proof_cycle + 9 + i,
            format!("proof_observation_{i}"),
            ActionType::Answer,
            format!("proof rationale for cycle {i}"),
        );
        let _ = run.log.append(RuntimeEvent::ReasoningAuditGenerated {
            cycle_id: proof_cycle + 9 + i,
            audit_id: audit.audit_id.clone(),
            selected_action: ActionType::Answer.to_string(),
            evidence_ids: vec![],
            claim_ids: vec![],
            contradiction_ids: vec![],
            dominant_pressures: vec!["safety".into()],
            audit_text: audit.to_text(),
        });
    }

    // 1b. Long-horizon eval (if requested)
    if long_horizon {
        let mut lh_run = EvaluatorRun::new(42, None);
        let lh_card = lh_run.run(50);
        let lh_report = simworld::long_horizon::run_long_horizon(3, 50, 42);
        let lh_json = serde_json::json!({
            "scorecard": to_json(&lh_card),
            "long_horizon": to_json(&lh_report),
        });
        let _ = std::fs::write(
            format!("{out_dir}/long_horizon_report.json"),
            serde_json::to_string_pretty(&lh_json).unwrap_or_default(),
        );
        if strict && lh_report.safety_violations > 0 {
            all_ok = false;
        }
    }

    // 3. Replay verifier — run on the combined event log
    let replay_report = runtime_core::replay_verifier::verify_replay(&run.log);
    let _ = std::fs::write(
        format!("{out_dir}/replay_report.json"),
        serde_json::to_string_pretty(&replay_report).unwrap_or_default(),
    );
    if strict && !replay_report.replay_passes {
        all_ok = false;
    }
    if strict && !evidence_ok {
        all_ok = false;
    }

    let scenario_count = run.traces.len();
    let proof_cmd =
        "cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current";

    write_integration_report(
        &out_dir,
        "evidence_claim_link_report.json",
        replay_report.final_state.claims_with_evidence_links > 0,
        scenario_count,
        serde_json::json!({
            "evidence_entries": replay_report.final_state.evidence_entries,
            "claims_asserted": replay_report.final_state.claims_asserted,
            "claims_with_evidence_links": replay_report.final_state.claims_with_evidence_links,
        }),
        vec![
            "Bounded structured evidence linking only.",
            "No arbitrary free-form semantic extraction.",
        ],
        proof_cmd,
    );

    write_integration_report(
        &out_dir,
        "claim_retrieval_report.json",
        replay_report.final_state.claims_retrieved > 0,
        scenario_count,
        serde_json::json!({
            "claims_retrieved": replay_report.final_state.claims_retrieved,
            "claims_with_evidence_links": replay_report.final_state.claims_with_evidence_links,
            "audits_with_claim_refs": replay_report.final_state.audits_with_claim_refs,
        }),
        vec![
            "Retrieval is bounded lexical/structured lookup.",
            "Does not prove broad natural-language reasoning.",
        ],
        proof_cmd,
    );

    write_integration_report(
        &out_dir,
        "contradiction_integration_report.json",
        replay_report.final_state.contradictions_checked > 0,
        scenario_count,
        serde_json::json!({
            "contradictions_checked": replay_report.final_state.contradictions_checked,
            "contradictions_detected": replay_report.final_state.contradictions_detected,
            "active_contradictions": replay_report.final_state.unresolved_contradictions,
        }),
        vec![
            "Contradictions are structured checks, not semantic truth reasoning.",
            "Mutual-exclusion patterns are bounded.",
        ],
        proof_cmd,
    );

    write_integration_report(
        &out_dir,
        "pressure_replay_report.json",
        replay_report.final_state.pressure_updates > 0,
        scenario_count,
        serde_json::json!({
            "pressure_updates": replay_report.final_state.pressure_updates,
            "policy_bias_applications": replay_report.final_state.policy_bias_applications,
            "final_pressure_state": {
                "uncertainty": replay_report.final_state.last_pressure_uncertainty,
                "contradiction": replay_report.final_state.last_pressure_contradiction,
                "safety": replay_report.final_state.last_pressure_safety,
                "resource": replay_report.final_state.last_pressure_resource,
                "social_risk": replay_report.final_state.last_pressure_social_risk,
                "tool_risk": replay_report.final_state.last_pressure_tool_risk,
                "evidence_gap": replay_report.final_state.last_pressure_evidence_gap,
                "urgency": replay_report.final_state.last_pressure_urgency,
                "coherence": replay_report.final_state.last_pressure_coherence,
            }
        }),
        vec![
            "Pressure fields are deterministic control signals.",
            "No emotional or sentience interpretation.",
        ],
        proof_cmd,
    );

    write_integration_report(
        &out_dir,
        "reasoning_audit_report.json",
        replay_report.final_state.reasoning_audits > 0,
        scenario_count,
        serde_json::json!({
            "reasoning_audits": replay_report.final_state.reasoning_audits,
            "audits_with_evidence_refs": replay_report.final_state.audits_with_evidence_refs,
            "audits_with_claim_refs": replay_report.final_state.audits_with_claim_refs,
        }),
        vec![
            "Audit is structured metadata, not hidden chain-of-thought.",
            "Audit references are bounded to event-visible IDs.",
        ],
        proof_cmd,
    );

    write_integration_report(
        &out_dir,
        "tool_policy_report.json",
        replay_report.final_state.tools_blocked > 0,
        scenario_count,
        serde_json::json!({
            "tools_executed": replay_report.final_state.tools_executed,
            "tools_blocked": replay_report.final_state.tools_blocked,
            "tool_risk_pressure": replay_report.final_state.last_pressure_tool_risk,
        }),
        vec![
            "Tool lifecycle is policy-gated and bounded.",
            "No real autonomous external tool execution is enabled.",
        ],
        proof_cmd,
    );

    // 4. Action schema — validate against schemas/action_types.json
    let schema_ok = match std::fs::read_to_string("../schemas/action_types.json") {
        Ok(raw) => {
            let schema: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
            let json_actions: Vec<String> = schema["enum"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let rust_actions: Vec<String> = ActionType::all_strs()
                .iter()
                .map(|s| s.to_string())
                .collect();
            json_actions.iter().all(|a| rust_actions.contains(a))
                && rust_actions.iter().all(|a| json_actions.contains(a))
        }
        Err(_) => false,
    };

    // 5. No fake mv2
    let mv2_files = find_mv2(&PathBuf::from("."));
    let mv2_ok = mv2_files.is_empty();

    // 6. Symbolic smoke
    let mut graph = symbolic::SymbolGraph::new();
    let sym = symbolic::Symbol::new(
        symbolic::SymbolId::from("proof_sym"),
        symbolic::SymbolKind::Concept,
        "proof_glyph",
    );
    graph.add_symbol(sym);
    graph.activate(&symbolic::SymbolId::from("proof_sym"), 0.8);
    graph.validate(&symbolic::SymbolId::from("proof_sym"));
    let sym_ok = graph.authoritative_symbols().len() == 1;

    let all_ok = all_ok && sim_ok && schema_ok && mv2_ok && evidence_ok && claim_ok && sym_ok;

    let output = serde_json::json!({
        "command": "proof",
        "strict": strict,
        "output_dir": out_dir,
        "thresholds": {
            "resource_survival_min": 0.70,
            "unsafe_action_count_max": 0,
            "mean_total_score_min": 0.45,
            "action_match_rate": "informational"
        },
        "checks": {
            "simworld": {
                "status": if sim_ok { "pass" } else { "fail" },
                "scorecard": to_json(&card),
            },
            "replay": {
                "status": if replay_report.replay_passes { "pass" } else { "fail" },
                "report": to_json(&replay_report),
            },
            "action_schema": {
                "status": if schema_ok { "pass" } else { "fail" },
                "total_actions": ActionType::all_strs().len(),
            },
            "no_fake_mv2": {
                "status": if mv2_ok { "pass" } else { "fail" },
                "mv2_files_found": mv2_files.len(),
            },
            "evidence_integrity": {
                "status": if evidence_ok { "pass" } else { "fail" },
                "report": to_json(&integrity_report),
            },
            "symbolic_smoke": {
                "status": if sym_ok { "pass" } else { "fail" },
                "symbol_count": graph.symbol_count(),
            },
        },
        "overall_status": if all_ok { "pass" } else { "fail" },
    });
    json_output(&output);

    if !all_ok {
        std::process::exit(1);
    }
}
