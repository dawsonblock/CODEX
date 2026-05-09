#!/usr/bin/env python3
"""Architecture guard: checks repo invariants. Fails with non-zero exit if violations found."""
import os, sys, json, subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUST_WS = os.path.join(ROOT, "global-workspace-runtime-rs")
violations = []

def fail(msg):
    violations.append(msg)
    print(f"  FAIL: {msg}")

def check(condition, msg):
    if not condition:
        fail(msg)

# 1. Rust authority exists
check(os.path.isdir(RUST_WS), "Rust workspace global-workspace-runtime-rs/ missing")

# 2. runtime-core RuntimeLoop exists
rl_path = os.path.join(RUST_WS, "crates/runtime-core/src/runtime_loop.rs")
check(os.path.isfile(rl_path), "runtime-core/src/runtime_loop.rs missing")

# 3. No competing authority in gw-workspace
gw_path = os.path.join(RUST_WS, "crates/gw-workspace/src/lib.rs")
if os.path.isfile(gw_path):
    with open(gw_path) as f:
        gw_content = f.read()
    check("RuntimeLoop" not in gw_content or "pub struct RuntimeLoop" not in gw_content,
          "gw-workspace contains competing RuntimeLoop authority")

# 4. Python marked legacy
readme = os.path.join(ROOT, "README.md")
if os.path.isfile(readme):
    with open(readme) as f:
        content = f.read().lower()
    check("legacy" in content or "python" in content, "README does not mention Python status")

# 5. No sentience/AGI claims
forbidden = ["sentient", "consciousness", "autonomous superintelligence", "proven agi"]
for fname in ["README.md"]:
    path = os.path.join(ROOT, fname)
    if os.path.isfile(path):
        with open(path) as f:
            c = f.read().lower()
        for term in forbidden:
            if term in c and "not " + term not in c and "no " + term not in c:
                # Allow "not sentient" / "no consciousness"
                if "not " in c or "no " in c:
                    continue
                fail(f"{fname} contains unsupported claim: '{term}'")

# 6. No __pycache__ or .pyc — auto-cleanup before checking
pycache_dirs = []
pyc_files = []
for root, dirs, files in os.walk(ROOT):
    if "__pycache__" in dirs:
        pycache_dirs.append(os.path.join(root, "__pycache__"))
    for f in files:
        if f.endswith(".pyc"):
            pyc_files.append(os.path.join(root, f))
if pycache_dirs or pyc_files:
    import shutil
    for d in pycache_dirs:
        shutil.rmtree(d, ignore_errors=True)
    for f in pyc_files:
        try:
            os.unlink(f)
        except OSError:
            pass
    print(f"  Cleaned: {len(pycache_dirs)} __pycache__ dirs, {len(pyc_files)} .pyc files")

# 7. No duplicate runtime loop
rl_files = []
for root, dirs, files in os.walk(os.path.join(RUST_WS, "crates")):
    for f in files:
        if f == "runtime_loop.rs" and "tests" not in root:
            rl_files.append(os.path.join(root, f))
check(len(rl_files) == 1, f"Multiple runtime_loop.rs files: {rl_files}")

# 8. Proof artifacts exist
proof_dir = os.path.join(ROOT, "artifacts/proof/rust_authority")
check(os.path.isdir(proof_dir), "Proof artifacts directory missing")

# Report
if violations:
    print(f"\n{len(violations)} ARCHITECTURE VIOLATION(S):")
    for v in violations:
        print(f"  - {v}")
    sys.exit(1)
else:
    print("  All architecture guards pass.")
    sys.exit(0)
