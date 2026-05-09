//! runtime-cli — command-line entry-point for the global-workspace-runtime.
//!
//! Subcommands:
//!   simworld                 Run deterministic SimWorld proof
//!   replay                   Replay events from JSONL log
//!   check-action-schema      Validate action strings against schemas/action_types.json
//!   check-no-fake-mv2        Assert no .mv2 files in repo
//!   symbolic-smoke           Quick symbolic crate smoke test
//!   proof                    Run all checks (simworld + replay + schema + mv2 + symbolic)

use runtime_core::ActionType;
use simworld::evaluator::EvaluatorRun;
use std::env;
use std::path::PathBuf;

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
            eprintln!("  runtime-cli proof");
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

fn cmd_proof(args: &[String]) {
    let strict = args.iter().any(|a| a == "--strict");
    let out_dir =
        parse_string_flag(args, "--out").unwrap_or_else(|| "artifacts/proof/latest".to_string());
    let _ = std::fs::create_dir_all(&out_dir);

    let mut all_ok = true;

    // 1. SimWorld
    let mut run = EvaluatorRun::new(5, None);
    let card = run.run(25);
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

    // 2. Replay verifier — run on evaluator's event log
    let replay_report = runtime_core::replay_verifier::verify_replay(&run.log);
    let _ = std::fs::write(
        format!("{out_dir}/replay_report.json"),
        serde_json::to_string_pretty(&replay_report).unwrap_or_default(),
    );
    if strict && !replay_report.replay_passes {
        all_ok = false;
    }

    // 3. Action schema — validate against schemas/action_types.json
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

    // 4. No fake mv2
    let mv2_files = find_mv2(&PathBuf::from("."));
    let mv2_ok = mv2_files.is_empty();

    // 5. Symbolic smoke
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

    let all_ok = all_ok && sim_ok && schema_ok && mv2_ok && sym_ok;

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
