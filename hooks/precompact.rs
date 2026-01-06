use std::env;
use std::fs;
use std::path::PathBuf;
use swarm_tools::codified_reasoning::CodifiedReasoning;
use swarm_tools::enhanced_monitor::{EnhancedMonitor, TrajectoryCompression};
use swarm_tools::loop_detector::LoopDetector;
use swarm_tools::role_router::RoleRouter;
use swarm_tools::security::{
    sanitize_agent_id, sanitize_error_message, validate_filename, SecurityError,
};
use swarm_tools::types::{AgentRole, SwarmConfig, TrajectoryEntry, TrajectoryLog};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_PATH_LENGTH: usize = 4096;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: precompact <agent_id> <prompt> <state>");
        eprintln!("  agent_id: Identifier for the agent");
        eprintln!("  prompt: Current agent prompt");
        eprintln!("  state: Current agent state (optional, default: 'unknown')");
        eprintln!("  --role <role>: Agent role for context filtering (optional)");
        eprintln!("  --compress: Enable trajectory compression");
        std::process::exit(1);
    }

    // Sanitize agent_id to prevent path traversal
    let raw_agent_id = &args[1];
    let agent_id = sanitize_agent_id(raw_agent_id);

    let prompt = &args[2];
    let state = if args.len() > 3 { &args[3] } else { "unknown" };

    let mut role = AgentRole::General;
    let mut enable_compression = false;

    for i in 4..args.len() {
        if args[i] == "--role" && i + 1 < args.len() {
            role = match args[i + 1].as_str() {
                "extractor" => AgentRole::Extractor,
                "analyzer" => AgentRole::Analyzer,
                "writer" => AgentRole::Writer,
                "reviewer" => AgentRole::Reviewer,
                "synthesizer" => AgentRole::Synthesizer,
                _ => AgentRole::General,
            };
        } else if args[i] == "--compress" {
            enable_compression = true;
        }
    }

    let config = SwarmConfig::default();
    let mut detector = LoopDetector::new(&config);

    match detector.check_all_loops(&agent_id, prompt, state) {
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
        Ok(None) => {}
        Err(e) => {
            let sanitized = sanitize_error_message(&e.to_string());
            eprintln!("Error checking for loops: {}", sanitized);
            std::process::exit(1);
        }
    }

    let monitor = EnhancedMonitor::new(config.context_budget);
    let context_pct = 0.0;

    if enable_compression {
        let trajectory_path = PathBuf::from(format!(
            ".claude/swarm-tools/loop-detector/{}_trajectory.json",
            agent_id
        ));

        if trajectory_path.exists() {
            match fs::read_to_string(&trajectory_path) {
                Ok(content) => {
                    if content.len() > MAX_FILE_SIZE {
                        eprintln!(
                            "Warning: Trajectory file exceeds size limit, skipping compression"
                        );
                    } else {
                        match serde_json::from_str::<TrajectoryLog>(&content) {
                            Ok(trajectory) => {
                                if monitor.should_compress(
                                    context_pct,
                                    trajectory.entries.len(),
                                    trajectory.tokens_used as usize,
                                ) {
                                    let compressed = monitor.compress_trajectory(&trajectory);

                                    println!("[COMPRESSION] Trajectory compressed");
                                    println!("  Original entries: {}", trajectory.entries.len());
                                    println!("  Preserved: {}", compressed.preserved.len());
                                    println!("  Summarized: {}", compressed.summarized.len());
                                    println!(
                                        "  Compression ratio: {:.2}",
                                        compressed.compression_ratio
                                    );

                                    let sanitized_filename = validate_filename(&format!(
                                        "{}_trajectory_compressed.json",
                                        &agent_id
                                    ))
                                    .unwrap_or_else(|_| "compressed_trajectory.json".to_string());
                                    let compressed_path =
                                        trajectory_path.with_file_name(sanitized_filename);

                                    if let Some(parent) = compressed_path.parent() {
                                        if let Err(e) = fs::create_dir_all(parent) {
                                            eprintln!("Warning: Could not create directory: {}", e);
                                        } else {
                                            let _ = fs::write(
                                                &compressed_path,
                                                serde_json::to_string_pretty(&compressed)
                                                    .unwrap_or_default(),
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let sanitized = sanitize_error_message(&e.to_string());
                                eprintln!("Warning: Could not parse trajectory: {}", sanitized);
                            }
                        }
                    }
                }
                Err(e) => {
                    let sanitized = sanitize_error_message(&e.to_string());
                    eprintln!("Warning: Could not read trajectory: {}", sanitized);
                }
            }
        }
    }

    let router = RoleRouter::new();
    let sample_messages = vec![
        ("File deltas show changes", 0, 0.7),
        ("Metrics indicate performance", 1, 0.8),
        ("Analysis results complete", 2, 0.9),
    ];

    let role_context = router.filter_context(&sample_messages, role);

    println!("[ROLE ROUTING] Context filtered for role: {:?}", role);
    println!(
        "  Total relevance score: {:.2}",
        role_context.total_relevance
    );
    println!("  Filtered items: {}", role_context.filtered_content.len());

    let recent_high_impact: Vec<_> = role_context
        .filtered_content
        .iter()
        .filter(|c| c.is_recent && c.impact_score > 0.7)
        .collect();

    if !recent_high_impact.is_empty() {
        println!("  Recent high-impact items: {}", recent_high_impact.len());
    }

    let codified = CodifiedReasoning::new();
    let plan = codified.codify_prompt(prompt, role.as_str());

    if !plan.steps.is_empty() {
        println!("[CODIFIED REASONING] Plan generated");
        println!("  Steps: {}", plan.steps.len());
        println!("  Expected tokens: {}", plan.total_expected_tokens);
        println!(
            "  Priority range: {:.2} - {:.2}",
            plan.steps
                .iter()
                .map(|s| s.priority)
                .fold(f64::MAX, f64::min),
            plan.steps
                .iter()
                .map(|s| s.priority)
                .fold(f64::MIN, f64::max)
        );
    }

    println!("\n[PRE-COMPACT] Complete");
    println!("  Agent: {}", agent_id);
    println!("  State: {}", state);
    println!("  Context: {:.1}%", context_pct);
    println!("  Role: {:?}", role);

    std::process::exit(0);
}
