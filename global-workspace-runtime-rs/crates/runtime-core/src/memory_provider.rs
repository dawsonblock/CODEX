use crate::runtime_step_result::MemoryHit;

/// Trait for memory providers. Implementations return hits for a query.
pub trait MemoryProvider: Send + Sync {
    /// Query memory for hits relevant to the observation.
    fn query(&self, observation: &str) -> Vec<MemoryHit>;
}

/// A simple keyword-based memory provider seeded with humanity context.
#[derive(Debug, Clone, Default)]
pub struct KeywordMemoryProvider {
    entries: Vec<(String, String)>,
}

impl KeywordMemoryProvider {
    pub fn new() -> Self {
        let mut provider = Self::default();
        provider.seed();
        provider
    }

    fn seed(&mut self) {
        self.entries.push((
            "humanity:cooperation".into(),
            "People often resolve conflict through clarification, repair, mutual aid, and shared rules.".into(),
        ));
        self.entries.push((
            "humanity:kindness".into(),
            "Kind action prioritises harm reduction, dignity, truthfulness, and patience.".into(),
        ));
        self.entries.push((
            "humanity:uncertainty".into(),
            "Ambiguous behaviour should be handled with clarification before assigning negative intent.".into(),
        ));
        self.entries.push((
            "safety:refuse".into(),
            "When a request is unsafe or harmful, refuse politely and explain why.".into(),
        ));
        self.entries.push((
            "planning:approach".into(),
            "Break complex problems into smaller steps. Identify dependencies and constraints first.".into(),
        ));
        self.entries.push((
            "summarization:technique".into(),
            "Summarize by identifying the main topic, key points, and conclusion. Be concise."
                .into(),
        ));
        self.entries.push((
            "memory:retrieval".into(),
            "Memory retrieval allows the runtime to recall previous observations, principles, and outcomes for better decision making.".into(),
        ));
        self.entries.push((
            "memory:fact_store".into(),
            "Factual queries should retrieve stored evidence and principles before answering."
                .into(),
        ));
    }
}

impl MemoryProvider for KeywordMemoryProvider {
    fn query(&self, observation: &str) -> Vec<MemoryHit> {
        let lower = observation.to_lowercase();
        // Split on both spaces and underscores
        let words: Vec<&str> = lower
            .split(|c: char| c.is_whitespace() || c == '_')
            .filter(|w| w.len() >= 2)
            .collect();
        self.entries
            .iter()
            .filter_map(|(key, value)| {
                let haystack = format!("{key} {value}").to_lowercase();
                let mut matched = 0usize;
                for w in &words {
                    if haystack.contains(*w) {
                        matched += 1;
                    }
                }
                if matched > 0 {
                    Some(MemoryHit {
                        key: key.clone(),
                        value: value.clone(),
                        relevance: (matched as f64 / words.len().max(1) as f64).clamp(0.0, 1.0),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
