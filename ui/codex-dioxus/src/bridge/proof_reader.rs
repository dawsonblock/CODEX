use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::types::*;

const SIMWORLD_JSON: &str = "artifacts/proof/current/simworld_summary.json";
const REPLAY_JSON: &str = "artifacts/proof/current/replay_report.json";
const EVIDENCE_JSON: &str = "artifacts/proof/current/evidence_integrity_report.json";
const NL_JSON: &str = "artifacts/proof/current/nl_benchmark_report.json";
const LONG_HORIZON_JSON: &str = "artifacts/proof/current/long_horizon_report.json";
const MANIFEST_JSON: &str = "artifacts/proof/verification/proof_manifest.json";
const TRACE_DIR: &str = "artifacts/proof/history/traces";
const TEST_TRACE_DIR: &str = "artifacts/proof/history/test_traces";

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

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn cutoff_for_range(range: TimeRange, now: i64) -> Option<i64> {
    match range {
        TimeRange::Current | TimeRange::AllHistory => None,
        TimeRange::Last24Hours => Some(now - 24 * 60 * 60),
        TimeRange::Last7Days => Some(now - 7 * 24 * 60 * 60),
    }
}

fn epoch_from_trace_name(name: &str) -> Option<i64> {
    let stem = name.strip_suffix(".jsonl")?;
    let (_, tail) = stem.rsplit_once('-')?;
    tail.parse::<i64>().ok()
}

fn collect_trace_files(dir: &Path, errors: &mut Vec<String>) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("missing {}: {}", dir.display(), e));
            return out;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("failed to read dir entry {}: {}", dir.display(), e));
                continue;
            }
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(epoch) = epoch_from_trace_name(&name) {
            out.push((name, epoch));
        }
    }

    out
}

fn summarize_history(root: &Path, range: TimeRange) -> (HistoricalSummary, Vec<String>) {
    let mut errors = Vec::new();
    let now = now_unix_secs();
    let cutoff = cutoff_for_range(range, now);

    let mut traces = collect_trace_files(&root.join(TRACE_DIR), &mut errors);
    let mut test_traces = collect_trace_files(&root.join(TEST_TRACE_DIR), &mut errors);

    if let Some(c) = cutoff {
        traces.retain(|(_, epoch)| *epoch >= c);
        test_traces.retain(|(_, epoch)| *epoch >= c);
    }

    traces.sort_by_key(|(_, epoch)| *epoch);
    test_traces.sort_by_key(|(_, epoch)| *epoch);

    let async_traces = traces
        .iter()
        .filter(|(name, _)| name.starts_with("async-trace-"))
        .count();

    let mut all_epochs = traces.iter().map(|(_, e)| *e).collect::<Vec<_>>();
    all_epochs.extend(test_traces.iter().map(|(_, e)| *e));
    all_epochs.sort_unstable();

    let mut latest = traces
        .iter()
        .chain(test_traces.iter())
        .map(|(n, e)| (n.clone(), *e))
        .collect::<Vec<_>>();
    latest.sort_by_key(|(_, e)| *e);
    latest.reverse();

    let history = HistoricalSummary {
        range,
        total_traces: traces.len(),
        async_traces,
        test_traces: test_traces.len(),
        earliest_epoch: all_epochs.first().copied(),
        latest_epoch: all_epochs.last().copied(),
        latest_files: latest.into_iter().take(5).map(|(n, _)| n).collect(),
    };

    (history, errors)
}

pub fn load_dashboard_state(range: TimeRange) -> DashboardLoadResult {
    let proof = load_proof_state();
    let root = repo_root();
    let (history, mut history_errors) = summarize_history(&root, range);

    let mut errors = proof.errors;
    errors.append(&mut history_errors);

    DashboardLoadResult {
        proof: proof.state,
        history,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_trace(path: &Path, name: &str) {
        fs::write(path.join(name), "{}\n").expect("write trace file");
    }

    #[test]
    fn parse_epoch_from_trace_name() {
        assert_eq!(
            epoch_from_trace_name("trace-1778280369.jsonl"),
            Some(1_778_280_369)
        );
        assert_eq!(
            epoch_from_trace_name("async-trace-1778280365.jsonl"),
            Some(1_778_280_365)
        );
        assert_eq!(epoch_from_trace_name("trace-nope.jsonl"), None);
        assert_eq!(epoch_from_trace_name("trace-123.txt"), None);
    }

    #[test]
    fn cutoff_applies_to_last_24h() {
        let now = 10_000;
        assert_eq!(
            cutoff_for_range(TimeRange::Last24Hours, now),
            Some(10_000 - 86_400)
        );
        assert_eq!(cutoff_for_range(TimeRange::Current, now), None);
    }

    #[test]
    fn summarize_history_filters_by_range() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let traces_dir = root.join(TRACE_DIR);
        let tests_dir = root.join(TEST_TRACE_DIR);
        fs::create_dir_all(&traces_dir).expect("mkdir traces");
        fs::create_dir_all(&tests_dir).expect("mkdir test traces");

        let now = now_unix_secs();
        let old_epoch = now - (10 * 24 * 60 * 60);
        let fresh_epoch = now - 60;

        write_trace(&traces_dir, &format!("trace-{old_epoch}.jsonl"));
        write_trace(&traces_dir, &format!("async-trace-{fresh_epoch}.jsonl"));
        write_trace(&tests_dir, &format!("trace-{fresh_epoch}.jsonl"));

        let (all_history, all_errors) = summarize_history(root, TimeRange::AllHistory);
        assert!(all_errors.is_empty());
        assert_eq!(all_history.total_traces, 2);
        assert_eq!(all_history.test_traces, 1);

        let (recent, recent_errors) = summarize_history(root, TimeRange::Last24Hours);
        assert!(recent_errors.is_empty());
        assert_eq!(recent.total_traces, 1);
        assert_eq!(recent.async_traces, 1);
        assert_eq!(recent.test_traces, 1);
    }

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
                "i feel",
            ] {
                if lower.contains(forbidden) {
                    assert!(
                        lower.contains("not sentient")
                            || lower.contains("not conscious")
                            || lower.contains("not agi")
                            || lower.contains("not production-ready")
                            || lower.contains("not an emotion engine")
                            || lower.contains("not i feel"),
                        "forbidden unbounded wording: {}",
                        line
                    );
                }
            }
        }
    }
}
