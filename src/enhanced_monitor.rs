use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHistoryEntry {
    pub tokens: usize,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopDetectionEvent {
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionEvent {
    pub success: bool,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeAdjustmentEvent {
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionEvent {
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailureEvent {
    pub error_type: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPercentageEntry {
    pub percentage: f64,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenVariance {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub max: usize,
    pub min: usize,
    pub range: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: String,
    pub agent_id: Option<String>,
    pub message: String,
    pub timestamp: String,
    pub extra: serde_json::Value,
}

pub struct EnhancedMonitor {
    #[allow(dead_code)]
    total_context: usize,

    agent_token_history: HashMap<String, VecDeque<TokenHistoryEntry>>,
    agent_token_rates: HashMap<String, f64>,

    loop_detection_rates: HashMap<String, Vec<LoopDetectionEvent>>,
    intervention_success_rates: HashMap<String, Vec<InterventionEvent>>,
    scope_adjustment_frequencies: HashMap<String, Vec<ScopeAdjustmentEvent>>,

    context_percentage_history: VecDeque<ContextPercentageEntry>,
    compaction_events: Vec<CompactionEvent>,
    agent_failures: HashMap<String, Vec<AgentFailureEvent>>,
    #[allow(dead_code)]
    overall_efficiency_history: VecDeque<f64>,

    #[allow(dead_code)]
    token_acceleration: HashMap<String, Vec<f64>>,
    context_trend: VecDeque<f64>,

    context_threshold: f64,
    variance_threshold: f64,
    acceleration_threshold: f64,
}

impl EnhancedMonitor {
    pub fn new(total_context: usize) -> Self {
        Self {
            total_context,
            agent_token_history: HashMap::new(),
            agent_token_rates: HashMap::new(),
            loop_detection_rates: HashMap::new(),
            intervention_success_rates: HashMap::new(),
            scope_adjustment_frequencies: HashMap::new(),
            context_percentage_history: VecDeque::with_capacity(1000),
            compaction_events: Vec::new(),
            agent_failures: HashMap::new(),
            overall_efficiency_history: VecDeque::with_capacity(100),
            token_acceleration: HashMap::new(),
            context_trend: VecDeque::with_capacity(100),
            context_threshold: 70.0,
            variance_threshold: 2.0,
            acceleration_threshold: 1000.0,
        }
    }

    pub fn record_token_usage(&mut self, agent_id: &str, tokens: usize, timestamp: Option<f64>) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        let history = self
            .agent_token_history
            .entry(agent_id.to_string())
            .or_insert_with(|| VecDeque::with_capacity(100));

        history.push_back(TokenHistoryEntry {
            tokens,
            timestamp: ts,
        });

        if history.len() > 100 {
            history.pop_front();
        }

        let history_vec: Vec<_> = history.iter().cloned().collect();
        if history_vec.len() >= 2 {
            let recent = if history_vec.len() >= 10 {
                &history_vec[history_vec.len() - 10..]
            } else {
                &history_vec
            };

            if let (Some(first), Some(last)) = (recent.first(), recent.last()) {
                let time_span = last.timestamp - first.timestamp;
                if time_span > 0.0 {
                    let token_change = last.tokens - first.tokens;
                    let rate = token_change as f64 / time_span;
                    self.agent_token_rates.insert(agent_id.to_string(), rate);
                }
            }
        }
    }

    pub fn record_context_percentage(&mut self, percentage: f64, timestamp: Option<f64>) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        self.context_percentage_history
            .push_back(ContextPercentageEntry {
                percentage,
                timestamp: ts,
            });

        if self.context_percentage_history.len() > 1000 {
            self.context_percentage_history.pop_front();
        }

        self.context_trend.push_back(percentage);
        if self.context_trend.len() > 100 {
            self.context_trend.pop_front();
        }
    }

    pub fn record_loop_detection(&mut self, agent_id: &str, timestamp: Option<f64>) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        let events = self
            .loop_detection_rates
            .entry(agent_id.to_string())
            .or_default();

        events.push(LoopDetectionEvent { timestamp: ts });
    }

    pub fn record_intervention(&mut self, agent_id: &str, success: bool, timestamp: Option<f64>) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        let events = self
            .intervention_success_rates
            .entry(agent_id.to_string())
            .or_default();

        events.push(InterventionEvent {
            success,
            timestamp: ts,
        });
    }

    pub fn record_scope_adjustment(&mut self, agent_id: &str, timestamp: Option<f64>) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        let events = self
            .scope_adjustment_frequencies
            .entry(agent_id.to_string())
            .or_default();

        events.push(ScopeAdjustmentEvent { timestamp: ts });
    }

    pub fn record_compaction(&mut self, timestamp: Option<f64>) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        self.compaction_events
            .push(CompactionEvent { timestamp: ts });
    }

    pub fn record_agent_failure(
        &mut self,
        agent_id: &str,
        error_type: &str,
        timestamp: Option<f64>,
    ) {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        });

        let events = self.agent_failures.entry(agent_id.to_string()).or_default();

        events.push(AgentFailureEvent {
            error_type: error_type.to_string(),
            timestamp: ts,
        });
    }

    pub fn get_token_variance(&self) -> Option<TokenVariance> {
        let mut current_tokens = Vec::new();

        for (agent_id, history) in &self.agent_token_history {
            if let Some(entry) = history.back() {
                current_tokens.push((agent_id.clone(), entry.tokens));
            }
        }

        if current_tokens.len() < 2 {
            return None;
        }

        let token_values: Vec<usize> = current_tokens.iter().map(|(_, t)| *t).collect();

        let mean = token_values.iter().map(|&t| t as f64).sum::<f64>() / token_values.len() as f64;

        let variance = token_values
            .iter()
            .map(|&t| {
                let diff = t as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / token_values.len() as f64;

        let std_dev = variance.sqrt();

        let max_token = *token_values.iter().max().unwrap_or(&0);
        let min_token = *token_values.iter().min().unwrap_or(&0);
        let range = max_token - min_token;

        Some(TokenVariance {
            mean,
            variance,
            std_dev,
            max: max_token,
            min: min_token,
            range,
        })
    }

    pub fn predict_context_overflow(&self) -> Option<PredictedOverflow> {
        if self.context_percentage_history.len() < 5 {
            return None;
        }

        let recent: Vec<_> = self
            .context_percentage_history
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        let percentages: Vec<f64> = recent.iter().map(|e| e.percentage).collect();
        let timestamps: Vec<f64> = recent.iter().map(|e| e.timestamp).collect();

        if percentages.len() >= 2 {
            let time_span = timestamps.last()? - timestamps.first()?;
            let percentage_change = percentages.last()? - percentages.first()?;

            if time_span > 0.0 {
                let rate = percentage_change / time_span;
                let current_pct = *percentages.last()?;

                if rate > 0.0 {
                    let to_threshold = self.context_threshold - current_pct;
                    let time_to_threshold = to_threshold / rate;

                    if time_to_threshold > 0.0 {
                        let overflow_time = timestamps.last()? + time_to_threshold;

                        return Some(PredictedOverflow {
                            current_percentage: current_pct,
                            rate: rate * 60.0,
                            time_to_threshold_seconds: time_to_threshold,
                            time_to_threshold_minutes: time_to_threshold / 60.0,
                            predicted_overflow_time: overflow_time,
                        });
                    }
                }
            }
        }

        None
    }

    pub fn check_token_variance_alert(&self) -> Option<Alert> {
        if let Some(variance) = self.get_token_variance() {
            for (agent_id, history) in &self.agent_token_history {
                if let Some(entry) = history.back() {
                    let current = entry.tokens;
                    let deviations_from_mean = if variance.std_dev > 0.0 {
                        (current as f64 - variance.mean).abs() / variance.std_dev
                    } else {
                        0.0
                    };

                    if deviations_from_mean > self.variance_threshold {
                        return Some(Alert {
                            alert_type: "high_token_variance".to_string(),
                            agent_id: Some(agent_id.clone()),
                            message: format!(
                                "Unusual token variance detected for agent {}: {} tokens vs mean {:.1} ({:.1} std devs)",
                                agent_id, current, variance.mean, deviations_from_mean
                            ),
                            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                            extra: serde_json::json!({
                                "current_tokens": current,
                                "mean_tokens": variance.mean,
                                "std_dev": variance.std_dev,
                                "deviations_from_mean": deviations_from_mean
                            }),
                        });
                    }
                }
            }
        }

        None
    }

    pub fn check_acceleration_alert(&self) -> Option<Alert> {
        for (agent_id, history) in &self.agent_token_history {
            if history.len() >= 5 {
                let history_vec: Vec<_> = history.iter().cloned().collect();
                let recent = &history_vec[history_vec.len() - 5..];

                let tokens: Vec<usize> = recent.iter().map(|e| e.tokens).collect();
                let timestamps: Vec<f64> = recent.iter().map(|e| e.timestamp).collect();

                if tokens.len() >= 3 {
                    let mut velocities = Vec::new();
                    for i in 1..tokens.len() {
                        let dt = timestamps[i] - timestamps[i - 1];
                        if dt > 0.0 {
                            velocities.push((tokens[i] - tokens[i - 1]) as f64 / dt);
                        }
                    }

                    if velocities.len() >= 2 {
                        let mut accelerations = Vec::new();
                        for i in 1..velocities.len() {
                            let dt = timestamps[i] - timestamps[i - 1];
                            if dt > 0.0 {
                                accelerations.push((velocities[i] - velocities[i - 1]) / dt);
                            }
                        }

                        if !accelerations.is_empty() {
                            let avg_acceleration =
                                accelerations.iter().sum::<f64>() / accelerations.len() as f64;

                            if avg_acceleration.abs() > self.acceleration_threshold {
                                return Some(Alert {
                                    alert_type: "token_acceleration".to_string(),
                                    agent_id: Some(agent_id.clone()),
                                    message: format!(
                                        "Token usage accelerating for agent {}: acceleration {:.1} tokens/s^2 indicates potential loop",
                                        agent_id, avg_acceleration
                                    ),
                                    timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                                    extra: serde_json::json!({
                                        "acceleration": avg_acceleration,
                                        "current_tokens": *tokens.last().unwrap_or(&0)
                                    }),
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn check_stagnation_alert(&self) -> Option<Alert> {
        let stagnation_threshold = 120.0;

        for (agent_id, history) in &self.agent_token_history {
            if history.len() >= 2 {
                let history_vec: Vec<_> = history.iter().cloned().collect();
                let recent = &history_vec[history_vec.len() - 2..];

                let time_diff = recent[1].timestamp - recent[0].timestamp;
                let token_diff = (recent[1].tokens as f64 - recent[0].tokens as f64).abs();

                if time_diff > stagnation_threshold && token_diff < 100.0 {
                    return Some(Alert {
                        alert_type: "agent_stagnation".to_string(),
                        agent_id: Some(agent_id.clone()),
                        message: format!(
                            "Agent {} stagnant for {:.0}s with only {:.0} token change - suggest guidance",
                            agent_id, time_diff, token_diff
                        ),
                        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                        extra: serde_json::json!({
                            "time_stagnant": time_diff,
                            "token_change": token_diff
                        }),
                    });
                }
            }
        }

        None
    }

    pub fn get_all_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();

        if let Some(variance_alert) = self.check_token_variance_alert() {
            alerts.push(variance_alert);
        }

        if let Some(acceleration_alert) = self.check_acceleration_alert() {
            alerts.push(acceleration_alert);
        }

        if let Some(stagnation_alert) = self.check_stagnation_alert() {
            alerts.push(stagnation_alert);
        }

        alerts
    }

    pub fn get_metrics_summary(&self) -> MetricsSummary {
        let token_variance = self.get_token_variance();

        let loop_detection_rates = self.calculate_loop_detection_rates();
        let intervention_success = self.calculate_intervention_success();

        let current_context = self
            .context_percentage_history
            .back()
            .map(|e| e.percentage)
            .unwrap_or(0.0);

        let compaction_count = self
            .compaction_events
            .iter()
            .filter(|e| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                now - e.timestamp < 3600.0
            })
            .count();

        MetricsSummary {
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            token_usage: token_variance,
            loop_detection_rates,
            intervention_success_rates: intervention_success,
            context_percentage: current_context,
            compactions_last_hour: compaction_count,
        }
    }

    fn calculate_loop_detection_rates(&self) -> HashMap<String, usize> {
        let mut rates = HashMap::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        for (agent_id, events) in &self.loop_detection_rates {
            let count = events.iter().filter(|e| now - e.timestamp < 3600.0).count();
            rates.insert(agent_id.clone(), count);
        }

        rates
    }

    fn calculate_intervention_success(&self) -> HashMap<String, f64> {
        let mut success_rates = HashMap::new();

        for (agent_id, events) in &self.intervention_success_rates {
            if !events.is_empty() {
                let successful = events.iter().filter(|e| e.success).count();
                let rate = (successful as f64 / events.len() as f64) * 100.0;
                success_rates.insert(agent_id.clone(), rate);
            }
        }

        success_rates
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedOverflow {
    pub current_percentage: f64,
    pub rate: f64,
    pub time_to_threshold_seconds: f64,
    pub time_to_threshold_minutes: f64,
    pub predicted_overflow_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub timestamp: String,
    pub token_usage: Option<TokenVariance>,
    pub loop_detection_rates: HashMap<String, usize>,
    pub intervention_success_rates: HashMap<String, f64>,
    pub context_percentage: f64,
    pub compactions_last_hour: usize,
}

impl Default for EnhancedMonitor {
    fn default() -> Self {
        Self::new(200_000)
    }
}
