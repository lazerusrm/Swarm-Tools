use serde::Deserialize;
use serde::Serialize;

use crate::types::{LoopDetection, LoopType, Result};
use hex::encode;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct LoopDetector {
    exact_loop_threshold: usize,
    semantic_loop_threshold: usize,
    state_oscillation_threshold: usize,
    semantic_similarity_threshold: f64,
    base_dir: PathBuf,
}

impl LoopDetector {
    pub fn new(config: &crate::types::SwarmConfig) -> Self {
        Self {
            exact_loop_threshold: config.loop_exact_threshold,
            semantic_loop_threshold: config.loop_semantic_threshold,
            state_oscillation_threshold: config.loop_state_oscillation_threshold,
            semantic_similarity_threshold: config.semantic_similarity_threshold,
            base_dir: PathBuf::from(".claude/swarm-tools"),
        }
    }

    fn hash_prompt(&self, prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        encode(hasher.finalize())
    }

    fn get_prompt_hashes_path(&self, agent_id: &str) -> PathBuf {
        self.base_dir
            .join("loop-detector")
            .join(format!("{}_hashes.json", agent_id))
    }

    fn get_prompt_history_path(&self, agent_id: &str) -> PathBuf {
        self.base_dir
            .join("loop-detector")
            .join(format!("{}_history.json", agent_id))
    }

    fn get_state_history_path(&self, agent_id: &str) -> PathBuf {
        self.base_dir
            .join("loop-detector")
            .join(format!("{}_state.json", agent_id))
    }

    fn load_hashes(&self, agent_id: &str) -> Result<HashMap<String, usize>> {
        let path = self.get_prompt_hashes_path(agent_id);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_hashes(&self, agent_id: &str, hashes: &HashMap<String, usize>) -> Result<()> {
        let path = self.get_prompt_hashes_path(agent_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(hashes)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn load_prompt_history(&self, agent_id: &str) -> Result<Vec<String>> {
        let path = self.get_prompt_history_path(agent_id);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Vec::new())
        }
    }

    fn save_prompt_history(&self, agent_id: &str, history: &Vec<String>) -> Result<()> {
        let path = self.get_prompt_history_path(agent_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(history)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn load_state_history(&self, agent_id: &str) -> Result<Vec<String>> {
        let path = self.get_state_history_path(agent_id);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Vec::new())
        }
    }

    fn save_state_history(&self, agent_id: &str, history: &Vec<String>) -> Result<()> {
        let path = self.get_state_history_path(agent_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(history)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn check_exact_loop(
        &mut self,
        agent_id: &str,
        prompt: &str,
    ) -> Result<Option<LoopDetection>> {
        let prompt_hash = self.hash_prompt(prompt);

        let mut hashes = self.load_hashes(agent_id)?;
        let count = hashes.get(&prompt_hash).copied().unwrap_or(0);
        hashes.insert(prompt_hash.clone(), count + 1);
        self.save_hashes(agent_id, &hashes)?;

        if count >= self.exact_loop_threshold {
            Ok(Some(LoopDetection {
                detection_type: LoopType::ExactLoop,
                agent_id: agent_id.to_string(),
                loop_count: count + 1,
                prompt_hash,
                timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            }))
        } else {
            Ok(None)
        }
    }

    fn semantic_similarity(&self, prompt1: &str, prompt2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = prompt1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = prompt2.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        intersection as f64 / union as f64
    }

    pub fn check_semantic_loop(
        &mut self,
        agent_id: &str,
        prompt: &str,
    ) -> Result<Option<LoopDetection>> {
        let history = self.load_prompt_history(agent_id)?;

        let mut similarity_count = 0;
        for hist_prompt in history.iter().rev().take(self.semantic_loop_threshold) {
            let similarity = self.semantic_similarity(prompt, hist_prompt);
            if similarity > self.semantic_similarity_threshold {
                similarity_count += 1;
            }
        }

        if similarity_count >= self.semantic_loop_threshold {
            Ok(Some(LoopDetection {
                detection_type: LoopType::SemanticLoop,
                agent_id: agent_id.to_string(),
                loop_count: similarity_count,
                prompt_hash: self.hash_prompt(prompt),
                timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn check_state_oscillation(
        &mut self,
        agent_id: &str,
        state: &str,
    ) -> Result<Option<LoopDetection>> {
        let mut history = self.load_state_history(agent_id)?;

        history.push(state.to_string());
        if history.len() > 20 {
            history.remove(0);
        }
        self.save_state_history(agent_id, &history)?;

        if history.len() >= self.state_oscillation_threshold * 2 {
            let recent = &history[history.len() - self.state_oscillation_threshold * 2..];

            let odd_states: Vec<String> = recent.iter().step_by(2).cloned().collect();
            let even_states: Vec<String> = recent.iter().skip(1).step_by(2).cloned().collect();

            let odd_set: std::collections::HashSet<String> = odd_states.iter().cloned().collect();
            let even_set: std::collections::HashSet<String> = even_states.iter().cloned().collect();

            if odd_set.len() == 1 && even_set.len() == 1 {
                let odd_state = odd_states.first().unwrap();
                let even_state = even_states.first().unwrap();

                if odd_state != even_state {
                    return Ok(Some(LoopDetection {
                        detection_type: LoopType::StateOscillation,
                        agent_id: agent_id.to_string(),
                        loop_count: self.state_oscillation_threshold,
                        prompt_hash: String::new(),
                        timestamp: chrono::Utc::now()
                            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    }));
                }
            }
        }

        Ok(None)
    }

    pub fn check_all_loops(
        &mut self,
        agent_id: &str,
        prompt: &str,
        state: &str,
    ) -> Result<Option<LoopDetection>> {
        let mut history = self.load_prompt_history(agent_id)?;
        history.push(prompt.to_string());
        if history.len() > 50 {
            history.remove(0);
        }
        self.save_prompt_history(agent_id, &history)?;

        let mut state_history = self.load_state_history(agent_id)?;
        state_history.push(state.to_string());
        if state_history.len() > 20 {
            state_history.remove(0);
        }
        self.save_state_history(agent_id, &state_history)?;

        if let Some(detection) = self.check_exact_loop(agent_id, prompt)? {
            return Ok(Some(detection));
        }

        if let Some(detection) = self.check_semantic_loop(agent_id, prompt)? {
            return Ok(Some(detection));
        }

        if let Some(detection) = self.check_state_oscillation(agent_id, state)? {
            return Ok(Some(detection));
        }

        Ok(None)
    }

    pub fn get_intervention_stats(&self) -> Result<InterventionStats> {
        let detector_dir = self.base_dir.join("loop-detector");
        let mut total_interventions: u64 = 0;
        let mut exact_loops: u64 = 0;
        let mut semantic_loops: u64 = 0;
        let mut state_oscillations: u64 = 0;

        if detector_dir.exists() {
            for entry in fs::read_dir(&detector_dir)? {
                let path = entry?.path();
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        let content = fs::read_to_string(&path)?;
                        let json: serde_json::Value = serde_json::from_str(&content)?;

                        if let Some(obj) = json.as_object() {
                            total_interventions += obj.len() as u64;

                            for (key, value) in obj {
                                if let Some(count) = value.as_u64() {
                                    if key.contains("exact") || count >= 3 {
                                        exact_loops += 1;
                                    } else if key.contains("semantic") || count >= 5 {
                                        semantic_loops += 1;
                                    } else if key.contains("oscillation") || count >= 3 {
                                        state_oscillations += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(InterventionStats {
            total_interventions,
            exact_loops,
            semantic_loops,
            state_oscillations,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterventionStats {
    pub total_interventions: u64,
    pub exact_loops: u64,
    pub semantic_loops: u64,
    pub state_oscillations: u64,
}
