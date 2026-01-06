use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: subagent_stop <agent_id> <state_file> <checkpoint_file>");
        eprintln!("  agent_id: Identifier for the agent");
        eprintln!("  state_file: Path to save agent state");
        eprintln!("  checkpoint_file: Path to save checkpoint");
        std::process::exit(1);
    }

    let agent_id = &args[1];
    let state_file = PathBuf::from(&args[2]);
    let checkpoint_file = PathBuf::from(&args[3]);

    println!("[STOP] Stopping subagent: {}", agent_id);

    let state_data = serde_json::json!({
        "agent_id": agent_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "stopped",
        "reason": "subagent_stop_hook"
    });

    if let Some(parent) = state_file.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Error creating directory: {}", e);
            std::process::exit(1);
        }
    }

    match fs::write(
        &state_file,
        serde_json::to_string_pretty(&state_data).unwrap(),
    ) {
        Ok(_) => println!("[STATE] Saved state to: {}", state_file.display()),
        Err(e) => {
            eprintln!("Error saving state: {}", e);
            std::process::exit(1);
        }
    }

    let checkpoint_data = serde_json::json!({
        "agent_id": agent_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checkpoint": true,
        "metadata": {
            "reason": "subagent_stop",
            "hooks_triggered": ["subagent_stop"]
        }
    });

    if let Some(parent) = checkpoint_file.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Error creating directory: {}", e);
            std::process::exit(1);
        }
    }

    match fs::write(
        &checkpoint_file,
        serde_json::to_string_pretty(&checkpoint_data).unwrap(),
    ) {
        Ok(_) => println!(
            "[CHECKPOINT] Saved checkpoint to: {}",
            checkpoint_file.display()
        ),
        Err(e) => {
            eprintln!("Error saving checkpoint: {}", e);
            std::process::exit(1);
        }
    }

    println!("[COMPLETE] Subagent {} stopped successfully", agent_id);
    std::process::exit(0);
}
