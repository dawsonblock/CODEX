#!/usr/bin/env python3
"""Thin wrapper delegating to scripts/architecture_guard.py.
Preserves exit code, CLI args, stdout, and stderr."""
import subprocess, sys, os

script = os.path.join(os.path.dirname(os.path.abspath(__file__)), "scripts", "architecture_guard.py")
result = subprocess.run([sys.executable, script, *sys.argv[1:]], capture_output=False)
sys.exit(result.returncode)
