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
- **MCP/Tool Routing** - Selective routing for Claude's Model Context Protocol tools to reduce token waste
- **Auto-Model Tiering** - Routes sub-tasks to appropriate Claude models (Haiku/Sonnet/Opus) based on complexity
- **Self-Healing Topology** - Automatically removes/reallocates low-contribution agents mid-swarm

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

## Configuration

### Built-in Features

All features can be configured via the feature configuration structs:

```rust
use swarm_tools::feature_config::{McpRoutingConfig, ModelTieringConfig, SelfHealingConfig};

let mcp_config = McpRoutingConfig {
    enabled: true,
    role_tool_filters: None, // Uses defaults
    default_tools: None,
};

let model_config = ModelTieringConfig {
    enabled: true,
    simple_haiku_threshold: 1000,
    moderate_sonnet_threshold: 5000,
    fallback_model: "claude-opus-4-5-2025".to_string(),
    high_impact_boost_enabled: true,
};

let healing_config = SelfHealingConfig {
    enabled: true,
    auto_prune_enabled: false, // Conservative by default
    prune_threshold: 0.3,
    prune_over_turns: 5,
    auto_rebalance_on_prune: true,
    min_active_agents: 2,
};
```

### Shared Configs

Pre-configured examples are available in `config_examples/`:

| File | Use Case |
|------|----------|
| `coding_swarm.json` | Optimized for code tasks with heavy extractor/analyzer tool filters |
| `research_swarm.json` | Web/browse tools enabled for synthesizer-focused research |
| `large_scale.json` | Higher parallelization with aggressive pruning settings |

#### Using Shared Configs

**Option 1: Copy to user config directory**

```bash
# Create config directory
mkdir -p ~/.claude/swarm-tools

# Copy desired config
cp config_examples/coding_swarm.json ~/.claude/swarm-tools/config_override.json
```

**Option 2: Environment variable**

```bash
export SWARM_TOOLS_CONFIG=/path/to/your/config.json
```

**Option 3: Merge with defaults**

The library will automatically look for configs in:
- `~/.claude/swarm-tools/config_override.json`
- `$XDG_CONFIG_HOME/swarm-tools/config.json` (on Linux/Mac)
- `%APPDATA%/swarm-tools/config.json` (on Windows)

## Model Tiering

The auto-model tiering feature routes tasks to appropriate Claude models:

| Tokens | Complexity | Default Model |
|--------|------------|---------------|
| < 1,000 | Simple | claude-haiku-4-5-2025 |
| 1,000 - 5,000 | Moderate | claude-sonnet-4-5-2025 |
| > 5,000 | Complex | claude-opus-4-5-2025 |

High-impact tasks (impact_score > 0.8) are automatically boosted to the next tier.

## Self-Healing Topology

Conservative by default (`auto_prune_enabled: false`). Enable for aggressive swarm optimization:

- Monitors contribution scores over turns
- Automatically prunes agents below threshold
- Reallocates budget to high-contribution agents
- Never prunes below `min_active_agents`

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
