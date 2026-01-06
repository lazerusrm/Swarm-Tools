# Swarm-Tools

Autonomous swarm optimization tools for detecting loops, optimizing prompts, and orchestrating multi-agent teams in Claude Code workflows.

## Features

- **Loop Detection** - Detects exact, semantic, and state oscillation loops with automatic intervention
- **OMAC Optimization** - Optimizes prompts and roles for conciseness and clarity
- **Team Optimization** - Composes and manages multi-agent teams with optimal workload distribution
- **Communication Optimization** - Filters redundant messages and prioritizes communication
- **Iterative Refinement** - Improves prompts through quality-based iterations
- **Parallel Execution** - Manages concurrent agent task execution
- **Enhanced Monitoring** - Tracks context usage, token rates, and intervention success

## Installation

```bash
cargo build --release
```

## Claude Code Plugin

Add to your `~/.claude/settings.json` plugins array:

```json
{
  "plugins": ["https://github.com/lazerusrm/Swarm-Tools"]
}
```

Or reference locally:

```json
{
  "plugins": ["/path/to/swarm-tools-rust"]
}
```

## Hooks Integration

### Precompact Hook
Run before context compaction to detect and intervene in loops:

```json
{
  "hooks": {
    "precompact": "./target/release/precompact.exe"
  }
}
```

### Subagent Stop Hook
Save agent state on stop:

```json
{
  "hooks": {
    "subagentStop": "./target/release/subagent_stop.exe"
  }
}
```

## Library Usage

```rust
use swarm_tools::loop_detector::LoopDetector;
use swarm_tools::types::SwarmConfig;

let config = SwarmConfig::default();
let mut detector = LoopDetector::new(&config);

if let Some(loop) = detector.check_all_loops("agent_1", prompt, state)? {
    // Handle detected loop
}
```

## Requirements

- Rust 1.70+
- Cargo

## License

MIT
