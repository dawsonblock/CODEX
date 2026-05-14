#!/usr/bin/env python3
"""Legacy shim — delegates to clean_generated_artifacts.py.

Kept for backward compatibility. Prefer calling clean_generated_artifacts.py
directly.
"""
import runpy, sys
from pathlib import Path

_script = Path(__file__).parent / "clean_generated_artifacts.py"
runpy.run_path(str(_script), run_name="__main__")
sys.exit(0)
