"""CI guard: assert ``ActionType`` Python enum and ``schemas/action_types.json``
define exactly the same set of action-type values.

This is a targeted schema-sync check rather than a whole-codebase string scan.
It ensures the JSON Schema artifact stays in sync with the Python enum so that
external consumers of the schema always see the correct vocabulary.

Usage::

    python -m global_workspace_runtime.scripts.check_action_types

Exits 0 on success, 1 on any violation.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from global_workspace_runtime.core.types import ActionType  # noqa: E402

_ROOT = Path(__file__).resolve().parents[3]
_SCHEMA_PATH = _ROOT / "schemas" / "action_types.json"


def main() -> None:
    # --- Load schema ---
    if not _SCHEMA_PATH.exists():
        print(f"FAIL: schema file not found at {_SCHEMA_PATH.relative_to(_ROOT)}")
        sys.exit(1)

    schema = json.loads(_SCHEMA_PATH.read_text(encoding="utf-8"))
    schema_values: set[str] = set(schema.get("enum", []))

    # --- Load enum ---
    enum_values: set[str] = {m.value for m in ActionType}

    # --- Compare ---
    missing_from_schema = enum_values - schema_values
    missing_from_enum = schema_values - enum_values

    if missing_from_schema or missing_from_enum:
        print("FAIL: ActionType enum and schema/action_types.json are out of sync.")
        if missing_from_schema:
            print(f"  In enum but NOT in schema: {sorted(missing_from_schema)}")
        if missing_from_enum:
            print(f"  In schema but NOT in enum: {sorted(missing_from_enum)}")
        sys.exit(1)

    print(
        f"PASS: ActionType enum and schema are in sync "
        f"({len(enum_values)} values: {sorted(enum_values)})"
    )


if __name__ == "__main__":
    main()

