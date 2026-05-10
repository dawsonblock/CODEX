"""Optional adapter for an external bayesian_brain.py router."""
from __future__ import annotations


class BayesianBrainAdapter:
    """Thin optional import wrapper.

    If a precision-weighted Bayesian capsule module is available on the Python
    path, this adapter can expose it. The main runtime does not require it.
    """

    def __init__(self) -> None:
        self.available = False
        self.module = None
        try:
            import bayesian_brain  # type: ignore
            self.module = bayesian_brain
            self.available = True
        except Exception:
            self.available = False

    def route_available(self) -> bool:
        return self.available
