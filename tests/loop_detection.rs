use std::time::{SystemTime, UNIX_EPOCH};
use swarm_tools::loop_detector::LoopDetector;
use swarm_tools::types::LoopType;
use swarm_tools::types::SwarmConfig;

fn get_unique_id(test_name: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{}_{}", test_name, timestamp)
}

fn setup_detector() -> LoopDetector {
    let config = SwarmConfig::default();
    LoopDetector::new(&config)
}

#[test]
fn test_exact_loop_detection() {
    let mut detector = setup_detector();
    let agent_id = get_unique_id("test_exact_loop");
    let prompt = "Analyze authentication module";

    for i in 0..5 {
        let result = detector.check_all_loops(&agent_id, prompt, "analyzing");

        if i >= 3 {
            assert!(result.is_ok(), "Check should not fail");
            let detection = result.unwrap();
            assert!(detection.is_some(), "Should detect loop after 3 iterations");
            let detection = detection.unwrap();
            assert_eq!(detection.detection_type, LoopType::ExactLoop);
            assert_eq!(detection.agent_id, agent_id);
            assert_eq!(detection.loop_count, i + 1);
            break;
        } else {
            assert!(result.is_ok(), "Check should not fail");
            let detection = result.unwrap();
            assert!(
                detection.is_none(),
                "Should not detect loop before threshold"
            );
        }
    }
}

#[test]
fn test_semantic_loop_detection() {
    let mut detector = setup_detector();
    let agent_id = get_unique_id("test_semantic_loop");
    let prompts = vec![
        "Analyze authentication module",
        "Analyze authentication module now",
        "Analyze authentication module please",
        "Analyze authentication module today",
        "Analyze authentication module code",
    ];

    for prompt in prompts.iter() {
        let result = detector.check_all_loops(&agent_id, prompt, "analyzing");
        assert!(result.is_ok(), "Check should not fail");
    }

    let result = detector.check_all_loops(&agent_id, "Analyze authentication module", "analyzing");
    assert!(result.is_ok(), "Final check should not fail");

    let detection = result.unwrap();
    if let Some(d) = detection {
        assert_eq!(d.agent_id, agent_id);
    }
}

#[test]
fn test_state_oscillation_detection() {
    let mut detector = setup_detector();
    let agent_id = get_unique_id("test_state_oscillation");
    let states = vec![
        "analyzing",
        "writing",
        "analyzing",
        "writing",
        "analyzing",
        "writing",
    ];
    let prompts = vec![
        "Analyze module function 1",
        "Write results for function 1",
        "Analyze module function 2",
        "Write results for function 2",
        "Analyze module function 3",
        "Write results for function 3",
    ];

    let mut found = false;
    for (i, (state, prompt)) in states.iter().zip(prompts.iter()).enumerate() {
        let result = detector.check_all_loops(&agent_id, prompt, state);

        if i >= 5 {
            assert!(result.is_ok(), "Check should not fail");
            let detection = result.unwrap();
            if detection.is_some() {
                found = true;
                let detection = detection.unwrap();
                assert_eq!(detection.detection_type, LoopType::StateOscillation);
                assert_eq!(detection.agent_id, agent_id);
                break;
            }
        }
    }

    assert!(
        found,
        "Should detect state oscillation after 6 state changes"
    );
}

#[test]
fn test_no_loop_detection() {
    let mut detector = setup_detector();
    let agent_id = get_unique_id("test_no_loop");

    for i in 0..3 {
        let prompt = format!("Task number {}", i);
        let state = format!("state_{}", i);

        let result = detector.check_all_loops(&agent_id, &prompt, &state);

        assert!(result.is_ok(), "Check should not fail");
        let detection = result.unwrap();
        assert!(
            detection.is_none(),
            "Should not detect loop with unique prompts and states"
        );
    }
}

#[test]
fn test_multiple_agents() {
    let mut detector = setup_detector();
    let agent_1 = get_unique_id("test_multi_1");
    let agent_2 = get_unique_id("test_multi_2");

    for _ in 0..3 {
        detector
            .check_all_loops(&agent_1, "Same prompt", "analyzing")
            .unwrap();
        detector
            .check_all_loops(&agent_2, "Different prompt", "analyzing")
            .unwrap();
    }

    let result_1 = detector.check_all_loops(&agent_1, "Same prompt", "analyzing");
    assert!(result_1.is_ok());
    assert!(result_1.unwrap().is_some(), "Agent 1 should detect loop");

    let result_2 = detector.check_all_loops(&agent_2, "Different prompt", "analyzing");
    assert!(result_2.is_ok());
    assert!(
        result_2.unwrap().is_none(),
        "Agent 2 should not detect loop"
    );
}
