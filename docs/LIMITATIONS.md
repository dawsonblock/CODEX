# Limitations

This project is a research scaffold for testing runtime architecture. It has
known limitations that must be stated honestly.

## Not production-ready

This code is not suitable for production deployment. It has not been hardened
for security, scalability, or reliability. No performance benchmarks exist.

## No real AGI

This project does not implement artificial general intelligence. The action
selection is deterministic and policy-driven. No learning, planning, or
reasoning beyond the scored action-set occurs.

## No sentience or consciousness

The numeric runtime variables (valence, arousal, threat, etc.) are engineering
metrics — they influence candidate scoring, nothing more. They do not represent
subjective experience, qualia, or consciousness. Any language suggesting
otherwise is an artifact of earlier project phases and has been removed.

## Memvid may be stubbed

`MemvidBackend` returns `NotImplemented` for all operations. No real Memvid
binary is integrated. Do not claim Memvid compatibility.

## Symbolic layer is bounded

The symbolic crate maintains a graph of concepts and principles. It is
internal abstraction machinery. Symbolic output is speculative unless
validated by the critic. The symbolic layer is not deeply wired into
RuntimeLoop selection — traces record what was activated, but the graph
does not directly drive decisions.

## SimWorld is synthetic

The SimWorld uses 7 deterministic scenario templates. It is a closed-world
simulation, not a real environment. Results should not be interpreted as
evidence of general capability.

## Performance score is not general intelligence

The `resource_survival`, `action_match_rate`, and `mean_total_score` metrics
measure specific runtime behavior in a synthetic environment. They are not
measures of intelligence, capability, or correctness in any general sense.

## Python is legacy

Python code under `src/global_workspace_runtime/` and `tests/` is maintained
for reference only. It does not validate Rust runtime behavior. No bridge
connects Python and Rust at runtime.

## License

MIT
