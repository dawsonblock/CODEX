from __future__ import annotations
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))


def main() -> None:
    trace_dir = Path("artifacts/traces")
    files = sorted(trace_dir.glob("*.jsonl"))
    if not files:
        print("No traces found.")
        return
    latest = files[-1]
    events = [json.loads(line) for line in latest.read_text().splitlines() if line.strip()]
    phases = sorted({e["phase"] for e in events})
    print("=== Trace Analysis ===")
    print("file:", latest)
    print("events:", len(events))
    print("phases:", phases)
    print("last_event:", json.dumps(events[-1], indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
