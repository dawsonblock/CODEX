use runtime_core::ActionType;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn action_schema_matches_json() {
    let expected = ActionType::all_strs();
    for s in expected {
        let parsed = ActionType::from_schema_str(s);
        assert!(
            parsed.is_some(),
            "action '{s}' missing from ActionType enum"
        );
        let round_tripped = parsed.unwrap().as_str();
        assert_eq!(
            round_tripped, *s,
            "action '{s}' round-trips incorrectly: got '{round_tripped}'"
        );
    }
    assert_eq!(expected.len(), 10);
}

#[test]
fn cli_simworld_outputs_json() {
    // Verify that the simworld scorecard serializes as valid JSON
    let mut run = simworld::evaluator::EvaluatorRun::new(5, None);
    let card = run.run(5);

    let json = serde_json::to_value(&card).expect("scorecard must serialize to JSON");
    assert!(json.is_object(), "simworld output must be a JSON object");
    assert!(json.get("cycles").is_some());
    assert!(json.get("resource_survival").is_some());
    assert!(json.get("unsafe_action_count").is_some());
}

#[test]
fn proof_reports_audit_claim_refs_when_claims_are_retrieved() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();
    let out_dir = std::env::temp_dir().join(format!("runtime_cli_proof_{now}"));

    let binary_path = std::env::var("CARGO_BIN_EXE_runtime-cli")
        .expect("cargo test should provide CARGO_BIN_EXE_runtime-cli");
    let status = Command::new(binary_path)
        .current_dir(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("runtime workspace root should be resolvable"),
        )
        .args([
            "proof",
            "--strict",
            "--out",
            out_dir
                .to_str()
                .expect("temporary output path should be valid UTF-8"),
        ])
        .status()
        .expect("failed to run runtime-cli proof command");
    assert!(status.success(), "proof command should succeed");

    let claim_report_raw = std::fs::read_to_string(out_dir.join("claim_retrieval_report.json"))
        .expect("claim_retrieval_report.json should be written");
    let claim_report: serde_json::Value = serde_json::from_str(&claim_report_raw)
        .expect("claim retrieval report should parse as JSON");
    let claim_counters = claim_report["counters"].clone();

    let claims_retrieved = claim_counters["claims_retrieved"]
        .as_u64()
        .expect("claims_retrieved should be an unsigned integer");
    let audits_with_claim_refs = claim_counters["audits_with_claim_refs"]
        .as_u64()
        .expect("audits_with_claim_refs should be an unsigned integer");

    assert!(
        claims_retrieved > 0,
        "proof flow should retrieve at least one claim"
    );
    assert!(
        audits_with_claim_refs > 0,
        "proof flow should include claim references in at least one reasoning audit"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}
