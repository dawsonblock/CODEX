#!/usr/bin/env python3
"""Clean stale Python artifacts: __pycache__ dirs and .pyc files."""
import os, sys, shutil

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

pycache_dirs = []
pyc_files = []
for root, dirs, files in os.walk(ROOT):
    if "__pycache__" in dirs:
        pycache_dirs.append(os.path.join(root, "__pycache__"))
    for f in files:
        if f.endswith(".pyc"):
            pyc_files.append(os.path.join(root, f))

for d in pycache_dirs:
    shutil.rmtree(d, ignore_errors=True)
for f in pyc_files:
    try:
        os.unlink(f)
    except OSError:
        pass

print(f"Cleaned: {len(pycache_dirs)} __pycache__ dirs, {len(pyc_files)} .pyc files")
