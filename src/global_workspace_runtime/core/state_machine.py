"""Seven-state runtime phase helper."""
from __future__ import annotations
from enum import Enum


class RuntimePhase(str, Enum):
    OBSERVE = "observe"
    EVALUATE = "evaluate"
    RECALL = "recall"
    COMPETE = "compete"
    GENERATE = "generate"
    SELECT = "select"
    CONSOLIDATE = "consolidate"


PHASE_OPTIONS = {
    RuntimePhase.OBSERVE: ("ignore", "encode"),
    RuntimePhase.EVALUATE: ("low_priority", "high_priority"),
    RuntimePhase.RECALL: ("normal_retrieval", "urgent_retrieval"),
    RuntimePhase.COMPETE: ("lose", "broadcast"),
    RuntimePhase.GENERATE: ("conservative_generation", "exploratory_generation"),
    RuntimePhase.SELECT: ("reject", "accept"),
    RuntimePhase.CONSOLIDATE: ("discard", "store"),
}
