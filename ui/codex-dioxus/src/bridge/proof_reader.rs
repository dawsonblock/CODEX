use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::types::*;

const SIMWORLD_JSON: &str = "artifacts/proof/current/simworld_summary.json";
const REPLAY_JSON: &str = "artifacts/proof/current/replay_report.json";
const EVIDENCE_JSON: &str = "artifacts/proof/current/evidence_integrity_report.json";
const NL_JSON: &str = "artifacts/proof/current/nl_benchmark_report.json";
const LONG_HORIZON_JSON: &str = "artifacts/proof/current/long_horizon_report.json";
const MANIFEST_JSON: &str = "artifacts/proof/verification/proof_manifest.json";

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn read_json<T: DeserializeOwned>(root: &Path, rel: &str) -> Result<T, String> {
    let path = root.join(rel);
    let raw =
        fs::read_to_string(&path).map_err(|e| format!("missing {}: {}", path.display(), e))?;
    serde_json::from_str::<T>(&raw).map_err(|e| format!("malformed {}: {}", path.display(), e))
}

fn read_nl_report(root: &Path) -> Result<NlBenchmarkReport, String> {
    let path = root.join(NL_JSON);
    let raw =
        fs::read_to_string(&path).map_err(|e| format!("missing {}: {}", path.display(), e))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("malformed {}: {}", path.display(), e))?;

    let sets = value
        .get("sets")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("malformed {}: missing sets array", path.display()))?;

    let mut report = NlBenchmarkReport::default();
    for entry in sets {
        if let Some(obj) = entry.as_object() {
            if let Some(v) = obj.get("curated") {
                report.curated = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("held_out") {
                report.held_out = serde_json::from_value(v.clone()).ok();
            }
            if let Some(v) = obj.get("adversarial") {
                report.adversarial = serde_json::from_value(v.clone()).ok();
            }
        }
    }

    Ok(report)
}

fn read_long_horizon(root: &Path) -> Result<LongHorizonReport, String> {
    let path = root.join(LONG_HORIZON_JSON);
    let raw =
        fs::read_to_string(&path).map_err(|e| format!("missing {}: {}", path.display(), e))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("malformed {}: {}", path.display(), e))?;

    if let Some(inner) = value.get("long_horizon") {
        serde_json::from_value(inner.clone())
            .map_err(|e| format!("malformed {}.long_horizon: {}", path.display(), e))
    } else {
        serde_json::from_value(value).map_err(|e| format!("malformed {}: {}", path.display(), e))
    }
}

pub fn load_proof_state() -> ProofLoadResult {
    let root = repo_root();
    let mut errors = Vec::new();

    let simworld = match read_json::<SimworldSummary>(&root, SIMWORLD_JSON) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            SimworldSummary::default()
        }
    };

    let replay = match read_json::<ReplayReport>(&root, REPLAY_JSON) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            ReplayReport::default()
        }
    };

    let evidence = match read_json::<EvidenceIntegrityReport>(&root, EVIDENCE_JSON) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            EvidenceIntegrityReport::default()
        }
    };

    let nl_benchmark = match read_nl_report(&root) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            NlBenchmarkReport::default()
        }
    };

    let long_horizon = match read_long_horizon(&root) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            LongHorizonReport::default()
        }
    };

    let manifest = match read_json::<ProofManifest>(&root, MANIFEST_JSON) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            ProofManifest::default()
        }
    };

    let state = CodexProofState {
        simworld,
        replay,
        evidence,
        nl_benchmark,
        long_horizon,
        manifest,
    };

    ProofLoadResult {
        state: Some(state),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_returns_error_not_panic() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let err = read_json::<SimworldSummary>(root, "missing.json").unwrap_err();
        assert!(err.contains("missing"));
    }

    #[test]
    fn malformed_json_returns_error() {
        let temp = tempfile::tempdir().expect("tempdir");
        let file = temp.path().join("bad.json");
        fs::write(&file, "{ not-json ").expect("write bad json");
        let err = read_json::<SimworldSummary>(temp.path(), "bad.json").unwrap_err();
        assert!(err.contains("malformed"));
    }

    #[test]
    fn schema_action_list_is_exactly_ten() {
        let actions = crate::components::action_schema_panel::ACTIONS;
        assert_eq!(actions.len(), 10);
        assert_eq!(actions[0], "answer");
        assert_eq!(actions[9], "internal_diagnostic");
    }

    #[test]
    fn ui_text_avoids_forbidden_claims() {
        let lines = crate::app::UI_BOUNDARY_LINES;
        for line in lines {
            let lower = line.to_lowercase();
            for forbidden in [
                "sentient",
                "conscious",
                "agi",
                "production-ready",
                "emotion engine",
            ] {
                if lower.contains(forbidden) {
                    assert!(
                        lower.contains("not sentient")
                            || lower.contains("not conscious")
                            || lower.contains("not agi")
                            || lower.contains("not production-ready")
                            || lower.contains("not an emotion engine"),
                        "forbidden unbounded wording: {}",
                        line
                    );
                }
            }
        }
    }
}
