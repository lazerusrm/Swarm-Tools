use std::env;
use std::fs;
use std::path::PathBuf;
use swarm_tools::codified_reasoning::CodifiedReasoning;
use swarm_tools::enhanced_monitor::{EnhancedMonitor, TrajectoryCompression};
use swarm_tools::security::{
    sanitize_agent_id, sanitize_error_message, validate_filename, SecurityError,
};
use swarm_tools::types::{Plan, TrajectoryEntry, TrajectoryLog};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_PATH_LENGTH: usize = 4096;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: subagent_stop <agent_id> <state_file> <checkpoint_file>");
        eprintln!("  agent_id: Identifier for the agent");
        eprintln!("  state_file: Path to save agent state");
        eprintln!("  checkpoint_file: Path to save checkpoint");
        eprintln!("  --plan <json>: Active plan to persist (optional)");
        eprintln!("  --trajectory <json>: Trajectory to persist (optional)");
        std::process::exit(1);
    }

    // Sanitize agent_id to prevent path traversal
    let raw_agent_id = &args[1];
    let agent_id = sanitize_agent_id(raw_agent_id);

    // Validate and sanitize file paths
    let state_file = match validate_filename(&args[2]) {
        Ok(name) => PathBuf::from(".claude/swarm-tools/states").join(name),
        Err(_) => {
            eprintln!("Error: Invalid state file path");
            std::process::exit(1);
        }
    };

    let checkpoint_file = match validate_filename(&args[3]) {
        Ok(name) => PathBuf::from(".claude/swarm-tools/checkpoints").join(name),
        Err(_) => {
            eprintln!("Error: Invalid checkpoint file path");
            std::process::exit(1);
        }
    };

    let mut active_plan: Option<Plan> = None;
    let mut trajectory_entries: Vec<TrajectoryEntry> = Vec::new();

    let mut i = 4;
    while i < args.len() {
        if args[i] == "--plan" && i + 1 < args.len() {
            let plan_json = &args[i + 1];
            // Limit JSON size to prevent DoS
            if plan_json.len() > MAX_FILE_SIZE {
                eprintln!("Warning: Plan JSON exceeds size limit, skipping");
            } else {
                match serde_json::from_str::<Plan>(plan_json) {
                    Ok(plan) => active_plan = Some(plan),
                    Err(e) => {
                        let sanitized = sanitize_error_message(&e.to_string());
                        eprintln!("Warning: Could not parse plan: {}", sanitized);
                    }
                }
            }
            i += 2;
        } else if args[i] == "--trajectory" && i + 1 < args.len() {
            let traj_json = &args[i + 1];
            // Limit JSON size to prevent DoS
            if traj_json.len() > MAX_FILE_SIZE {
                eprintln!("Warning: Trajectory JSON exceeds size limit, skipping");
            } else {
                match serde_json::from_str::<Vec<TrajectoryEntry>>(traj_json) {
                    Ok(entries) => trajectory_entries = entries,
                    Err(e) => {
                        let sanitized = sanitize_error_message(&e.to_string());
                        eprintln!("Warning: Could not parse trajectory: {}", sanitized);
                    }
                }
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    println!("[STOP] Stopping subagent: {}", agent_id);

    let timestamp = chrono::Utc::now().to_rfc3339();

    let mut state_obj = serde_json::Map::new();
    state_obj.insert(
        "agent_id".to_string(),
        serde_json::Value::String(agent_id.clone()),
    );
    state_obj.insert(
        "timestamp".to_string(),
        serde_json::Value::String(timestamp.clone()),
    );
    state_obj.insert(
        "status".to_string(),
        serde_json::Value::String("stopped".to_string()),
    );
    state_obj.insert(
        "reason".to_string(),
        serde_json::Value::String("subagent_stop_hook".to_string()),
    );

    if let Some(plan) = &active_plan {
        state_obj.insert(
            "codified_plan".to_string(),
            serde_json::to_value(plan).unwrap_or_default(),
        );
        println!(
            "[PLAN] Active plan with {} steps persisted",
            plan.steps.len()
        );
    }

    let state_data = serde_json::Value::Object(serde_json::Map::from(state_obj));

    if let Some(parent) = state_file.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            let sanitized = sanitize_error_message(&e.to_string());
            eprintln!("Error creating directory: {}", sanitized);
            std::process::exit(1);
        }
    }

    match fs::write(
        &state_file,
        serde_json::to_string_pretty(&state_data).unwrap_or_default(),
    ) {
        Ok(_) => println!("[STATE] Saved state to: {}", state_file.display()),
        Err(e) => {
            let sanitized = sanitize_error_message(&e.to_string());
            eprintln!("Error saving state: {}", sanitized);
            std::process::exit(1);
        }
    }

    let trajectory = TrajectoryLog {
        entries: trajectory_entries.clone(),
        tokens_used: trajectory_entries.iter().map(|e| e.tokens_used).sum(),
        compressibility_score: EnhancedMonitor::default().get_compression_threshold().0 as f64
            / trajectory_entries.len().max(1) as f64,
        created_at: timestamp.clone(),
    };

    let trajectory_path = PathBuf::from(format!(
        ".claude/swarm-tools/loop-detector/{}_trajectory.json",
        agent_id
    ));

    if let Some(parent) = trajectory_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            let sanitized = sanitize_error_message(&e.to_string());
            eprintln!(
                "Warning: Could not create trajectory directory: {}",
                sanitized
            );
        } else {
            match fs::write(
                &trajectory_path,
                serde_json::to_string_pretty(&trajectory).unwrap_or_default(),
            ) {
                Ok(_) => println!("[TRAJECTORY] Saved {} entries", trajectory.entries.len()),
                Err(e) => {
                    let sanitized = sanitize_error_message(&e.to_string());
                    eprintln!("Warning: Could not save trajectory: {}", sanitized);
                }
            }
        }
    }

    let mut checkpoint_obj = serde_json::Map::new();
    checkpoint_obj.insert(
        "agent_id".to_string(),
        serde_json::Value::String(agent_id.clone()),
    );
    checkpoint_obj.insert(
        "timestamp".to_string(),
        serde_json::Value::String(timestamp.clone()),
    );
    checkpoint_obj.insert("checkpoint".to_string(), serde_json::Value::Bool(true));

    let mut metadata = serde_json::Map::new();
    metadata.insert(
        "reason".to_string(),
        serde_json::Value::String("subagent_stop".to_string()),
    );
    metadata.insert(
        "hooks_triggered".to_string(),
        serde_json::Value::Array(vec![serde_json::Value::String("subagent_stop".to_string())]),
    );
    metadata.insert(
        "trajectory_entries".to_string(),
        serde_json::Value::Number(serde_json::Number::from(trajectory_entries.len())),
    );

    if let Some(plan) = &active_plan {
        metadata.insert(
            "plan_steps".to_string(),
            serde_json::Value::Number(serde_json::Number::from(plan.steps.len())),
        );
        metadata.insert(
            "plan_tokens".to_string(),
            serde_json::Value::Number(serde_json::Number::from(plan.total_expected_tokens)),
        );
    }

    checkpoint_obj.insert("metadata".to_string(), serde_json::Value::Object(metadata));

    let checkpoint_data = serde_json::Value::Object(checkpoint_obj);

    if let Some(parent) = checkpoint_file.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            let sanitized = sanitize_error_message(&e.to_string());
            eprintln!("Error creating directory: {}", sanitized);
            std::process::exit(1);
        }
    }

    match fs::write(
        &checkpoint_file,
        serde_json::to_string_pretty(&checkpoint_data).unwrap_or_default(),
    ) {
        Ok(_) => println!(
            "[CHECKPOINT] Saved checkpoint to: {}",
            checkpoint_file.display()
        ),
        Err(e) => {
            let sanitized = sanitize_error_message(&e.to_string());
            eprintln!("Error saving checkpoint: {}", sanitized);
            std::process::exit(1);
        }
    }

    println!("\n[STOP SUMMARY] Agent: {}", agent_id);
    println!("  State saved: {}", state_file.display());
    println!("  Checkpoint saved: {}", checkpoint_file.display());
    println!("  Trajectory entries: {}", trajectory_entries.len());

    if let Some(plan) = &active_plan {
        println!(
            "  Plan steps: {} ({} tokens estimated)",
            plan.steps.len(),
            plan.total_expected_tokens
        );
    }

    println!("\n[COMPLETE] Subagent {} stopped successfully", agent_id);
    std::process::exit(0);
}
