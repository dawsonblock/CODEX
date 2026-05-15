# EventOrigin / EventEnvelope Status

## Implemented

- EventOrigin enum includes expanded subsystem variants.
- EventEnvelope persists event origin metadata.
- `append_with_origin` is available for explicit subsystem attribution.

## Current limitation

- Call-site adoption is partial.
- Some call sites still use `append()` and therefore default origin to `RuntimeLoop`.
- This package does not claim perfect subsystem attribution yet.

## Evidence reference

See `artifacts/proof/current/event_log_sequence_report.json` (notes include `original_append_callsites` and partial adoption caveats).
