"""Persistent runtime state."""
from __future__ import annotations
from dataclasses import dataclass, field
from .types import InternalState, GoalState, ResourceState
from .event_log import EventLog
from ..memory import EpisodicMemory, SemanticMemory, SemanticCache, Scratchpad, SelfModel, ConsolidationQueue, JsonlArchive, MemoryAbstractor
from ..modulation.somatic import SomaticMap


@dataclass
class RuntimeState:
    internal_state: InternalState = field(default_factory=InternalState)
    goal_state: GoalState = field(default_factory=GoalState)
    resource_state: ResourceState = field(default_factory=ResourceState)
    episodic_memory: EpisodicMemory = field(default_factory=EpisodicMemory)
    semantic_memory: SemanticMemory = field(default_factory=SemanticMemory)
    semantic_cache: SemanticCache = field(default_factory=SemanticCache)
    scratchpad: Scratchpad = field(default_factory=Scratchpad)
    self_model: SelfModel = field(default_factory=SelfModel)
    consolidation_queue: ConsolidationQueue = field(default_factory=ConsolidationQueue)
    long_term_archive: JsonlArchive = field(default_factory=JsonlArchive)
    memory_abstractor: MemoryAbstractor = field(default_factory=MemoryAbstractor)
    somatic_map: SomaticMap = field(default_factory=SomaticMap)
    event_log: EventLog = field(default_factory=lambda: EventLog(None))
    cycle_id: int = 0

    def next_cycle(self) -> int:
        self.cycle_id += 1
        return self.cycle_id
