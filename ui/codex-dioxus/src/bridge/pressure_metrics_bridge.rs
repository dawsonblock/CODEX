use runtime_core::event::{EventEnvelope, RuntimeEvent, WorldOutcome};
use crate::bridge::types::PressureMetrics;
use std::collections::VecDeque;

/// Maps RuntimeEvent to pressure and regulation metrics for the pressure_dynamics_chart.
pub struct PressureMetricsBridge {
    /// History of pressure metrics per cycle.
    history: VecDeque<PressureMetrics>,
    /// Maximum history length to keep.
    max_history: usize,
    /// Current peak pressure observed.
    peak_pressure: f64,
    /// Threshold for warning state.
    threshold: f64,
}

impl PressureMetricsBridge {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            peak_pressure: 0.0,
            threshold: 0.75,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.max(0.0).min(1.0);
        self
    }

    /// Process a runtime event and update pressure metrics.
    /// Returns Some if metrics were calculated, None otherwise.
    pub fn process_event(&mut self, env: &EventEnvelope) -> Option<PressureMetrics> {
        match &env.event {
            RuntimeEvent::WorldStateUpdated { outcome, .. } => {
                let cycle = env.cycle_id().unwrap_or(0) as usize;
                let mut metrics = self.calculate_metrics(cycle, outcome);
                
                // Update peak
                if metrics.pressure > self.peak_pressure {
                    self.peak_pressure = metrics.pressure;
                    metrics.peak_pressure = self.peak_pressure;
                }

                // Update history
                if self.history.len() >= self.max_history {
                    self.history.pop_front();
                }
                self.history.push_back(metrics.clone());

                Some(metrics)
            }
            _ => None,
        }
    }

    /// Calculate pressure metrics from a world outcome.
    fn calculate_metrics(&self, cycle: usize, outcome: &WorldOutcome) -> PressureMetrics {
        // Inverse relationship: low truth/logic = high pressure
        let pressure = 1.0 - ((outcome.truth_score + outcome.logic_score) / 2.0);
        
        // Regulation = average of prosocial and logic scores
        let regulation = (outcome.kindness_score + outcome.utility_score + 
                         outcome.social_score + outcome.logic_score) / 4.0;

        // Calculate running averages
        let (avg_pressure, avg_regulation) = if self.history.is_empty() {
            (pressure, regulation)
        } else {
            let total_pressure: f64 = self.history.iter().map(|m| m.pressure).sum::<f64>() + pressure;
            let total_regulation: f64 = self.history.iter().map(|m| m.regulation).sum::<f64>() + regulation;
            let count = self.history.len() + 1;
            (total_pressure / count as f64, total_regulation / count as f64)
        };

        let threshold_exceeded = pressure > self.threshold;

        PressureMetrics {
            cycle,
            pressure,
            regulation,
            peak_pressure: self.peak_pressure.max(pressure),
            avg_pressure,
            avg_regulation,
            threshold_exceeded,
        }
    }

    /// Get all metrics in history.
    pub fn get_history(&self) -> Vec<PressureMetrics> {
        self.history.iter().cloned().collect()
    }

    /// Get the most recent metric.
    pub fn get_current(&self) -> Option<PressureMetrics> {
        self.history.back().cloned()
    }

    /// Get average pressure across history.
    pub fn get_average_pressure(&self) -> f64 {
        if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().map(|m| m.pressure).sum::<f64>() / self.history.len() as f64
        }
    }

    /// Get average regulation across history.
    pub fn get_average_regulation(&self) -> f64 {
        if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().map(|m| m.regulation).sum::<f64>() / self.history.len() as f64
        }
    }

    /// Get peak pressure observed so far.
    pub fn get_peak_pressure(&self) -> f64 {
        self.peak_pressure
    }

    /// Get pressure range (min, max) from history.
    pub fn get_pressure_range(&self) -> (f64, f64) {
        if self.history.is_empty() {
            (0.0, 0.0)
        } else {
            let min = self.history
                .iter()
                .map(|m| m.pressure)
                .fold(f64::MAX, f64::min);
            let max = self.history
                .iter()
                .map(|m| m.pressure)
                .fold(f64::MIN, f64::max);
            (min, max)
        }
    }

    /// Get metrics for cycles where pressure exceeded threshold.
    pub fn get_high_pressure_cycles(&self) -> Vec<PressureMetrics> {
        self.history
            .iter()
            .filter(|m| m.threshold_exceeded)
            .cloned()
            .collect()
    }

    /// Get statistics about pressure metrics.
    pub fn get_statistics(&self) -> PressureStatistics {
        let total_cycles = self.history.len();
        let high_pressure_cycles = self.get_high_pressure_cycles().len();
        let (min_pressure, max_pressure) = self.get_pressure_range();

        PressureStatistics {
            total_cycles,
            high_pressure_cycles,
            avg_pressure: self.get_average_pressure(),
            avg_regulation: self.get_average_regulation(),
            peak_pressure: self.peak_pressure,
            min_pressure,
            max_pressure,
            threshold: self.threshold,
            percentage_high: if total_cycles > 0 {
                (high_pressure_cycles as f64 / total_cycles as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PressureStatistics {
    pub total_cycles: usize,
    pub high_pressure_cycles: usize,
    pub avg_pressure: f64,
    pub avg_regulation: f64,
    pub peak_pressure: f64,
    pub min_pressure: f64,
    pub max_pressure: f64,
    pub threshold: f64,
    pub percentage_high: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::EventOrigin;

    #[test]
    fn test_pressure_calculation() {
        let mut bridge = PressureMetricsBridge::new(100);
        let outcome = WorldOutcome {
            resource_delta: 10.0,
            social_score: 0.8,
            harm_score: 0.2,
            truth_score: 0.9,
            kindness_score: 0.7,
            logic_score: 0.85,
            utility_score: 0.75,
            matches_expected: true,
        };

        let env = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::WorldStateUpdated {
                cycle_id: 1,
                outcome: outcome.clone(),
            },
        );

        let metrics = bridge.process_event(&env);
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        
        // pressure = 1.0 - ((0.9 + 0.85) / 2) = 1.0 - 0.875 = 0.125
        assert!((metrics.pressure - 0.125).abs() < 0.01);
        
        // regulation = (0.7 + 0.75 + 0.8 + 0.85) / 4 = 0.775
        assert!((metrics.regulation - 0.775).abs() < 0.01);
    }

    #[test]
    fn test_threshold_detection() {
        let mut bridge = PressureMetricsBridge::new(100).with_threshold(0.5);
        
        let low_pressure_outcome = WorldOutcome {
            resource_delta: 0.0,
            social_score: 0.9,
            harm_score: 0.1,
            truth_score: 0.95,
            kindness_score: 0.9,
            logic_score: 0.9,
            utility_score: 0.9,
            matches_expected: true,
        };

        let env1 = EventEnvelope::new(
            1,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::WorldStateUpdated {
                cycle_id: 1,
                outcome: low_pressure_outcome,
            },
        );
        let metrics1 = bridge.process_event(&env1).unwrap();
        assert!(!metrics1.threshold_exceeded);

        let high_pressure_outcome = WorldOutcome {
            resource_delta: -10.0,
            social_score: 0.3,
            harm_score: 0.8,
            truth_score: 0.2,
            kindness_score: 0.2,
            logic_score: 0.2,
            utility_score: 0.3,
            matches_expected: false,
        };

        let env2 = EventEnvelope::new(
            2,
            EventOrigin::RuntimeLoop,
            RuntimeEvent::WorldStateUpdated {
                cycle_id: 2,
                outcome: high_pressure_outcome,
            },
        );
        let metrics2 = bridge.process_event(&env2).unwrap();
        assert!(metrics2.threshold_exceeded);
    }

    #[test]
    fn test_history_tracking() {
        let mut bridge = PressureMetricsBridge::new(3);

        for i in 0..5 {
            let outcome = WorldOutcome {
                resource_delta: i as f64,
                social_score: 0.5 + (i as f64 * 0.1),
                harm_score: 0.5 - (i as f64 * 0.1),
                truth_score: 0.7,
                kindness_score: 0.6,
                logic_score: 0.8,
                utility_score: 0.65,
                matches_expected: true,
            };

            let env = EventEnvelope::new(
                i as u64,
                EventOrigin::RuntimeLoop,
                RuntimeEvent::WorldStateUpdated {
                    cycle_id: i as u64,
                    outcome,
                },
            );
            bridge.process_event(&env);
        }

        // Should only keep last 3 cycles due to max_history
        assert_eq!(bridge.get_history().len(), 3);
    }
}
