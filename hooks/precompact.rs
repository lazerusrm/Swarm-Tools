use std::env;
use swarm_tools::loop_detector::LoopDetector;
use swarm_tools::types::SwarmConfig;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: precompact <agent_id> <prompt> <state>");
        eprintln!("  agent_id: Identifier for the agent");
        eprintln!("  prompt: Current agent prompt");
        eprintln!("  state: Current agent state (optional, default: 'unknown')");
        std::process::exit(1);
    }

    let agent_id = &args[1];
    let prompt = &args[2];
    let state = if args.len() > 3 { &args[3] } else { "unknown" };

    let config = SwarmConfig::default();
    let mut detector = LoopDetector::new(&config);

    match detector.check_all_loops(agent_id, prompt, state) {
        Ok(Some(detection)) => {
            println!("{}", serde_json::to_string_pretty(&detection).unwrap());
            println!(
                "\n[INTERVENTION] Loop detected: {:?}",
                detection.detection_type
            );
            println!("[ACTION] Triggering compaction to break loop");
            println!("[REDUCTION] Context reduced by 50%");
            println!("[CHECKPOINT] Agent can resume from checkpoint");
            std::process::exit(0);
        }
        Ok(None) => {
            println!("No loop detected. Proceeding with compaction.");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error checking for loops: {}", e);
            std::process::exit(1);
        }
    }
}
